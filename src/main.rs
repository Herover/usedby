use clap::{arg, value_parser, ArgAction, Command};
use procfs::{
    net::TcpState,
    process::{FDTarget, Stat},
};
use std::collections::HashMap;

struct ProcessInfo {
    pid: i32,
    cmd: String,
    exe: String,
    parent_pid: i32,
    uid: Option<u32>,
}

const UNKNOWN_INDICATOR: &str = "?";

fn cli() -> Command {
    Command::new("usedby")
        .about("A fictional versioning CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("port")
                .about("Scans for processes currently using a port")
                .arg(
                    arg!(<PORT> "Port number")
                        .value_parser(clap::value_parser!(u16).range(1..65535))
                        .value_parser(value_parser!(usize))
                        .action(ArgAction::Set),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("file")
                .about("Scans for processes that currently have a file open")
                .arg(
                    arg!(<FILE> "File"), /* .value_parser(clap::value_parser!(PathBuf)) */
                )
                .arg_required_else_help(true),
        )
}

fn push_args() -> Vec<clap::Arg> {
    vec![arg!(-m --message <MESSAGE>)]
}

fn main() {
    let all_procs = procfs::process::all_processes().unwrap();

    // build up a map between socket inodes and process stat info:
    let mut inode_map: HashMap<u64, Stat> = HashMap::new();
    let mut process_map: HashMap<i32, ProcessInfo> = HashMap::new();
    for p in all_procs {
        if let Ok(process) = p {
            let mut ppid = 0;
            if let (Ok(stat), Ok(fds)) = (process.stat(), process.fd()) {
                ppid = stat.ppid;
                for fd in fds {
                    match fd.unwrap().target {
                        FDTarget::Socket(inode) => inode_map.insert(inode, stat.clone()),
                        _ => None,
                    };
                }
            }
            let mut exe = String::from(UNKNOWN_INDICATOR);
            if let Ok(str) = process.exe() {
                exe = str.to_str().unwrap().to_string();
            }
            process_map.insert(
                process.pid,
                ProcessInfo {
                    pid: process.pid,
                    cmd: process.cmdline().unwrap().join(" "),
                    exe: exe,
                    parent_pid: ppid,
                    uid: process.uid().map_or(None, |v| Some(v)),
                },
            );
        }
    }

    let mut is_first = true;

    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("port", sub_matches)) => {
            let port_number = sub_matches.get_one::<usize>("PORT").unwrap();

            print_header();
            let tcp = procfs::net::tcp().unwrap();
            let tcp6 = procfs::net::tcp6().unwrap();
            for entry in tcp.into_iter().chain(tcp6) {
                if usize::from(entry.local_address.port()) == *port_number
                    && entry.state == TcpState::Listen
                {
                    if !is_first {
                        println!();
                    }
                    is_first = false;

                    if let Some(stat) = inode_map.get(&entry.inode) {
                        let mut processes = get_process_parents(stat.pid, &inode_map, &process_map);
                        processes.reverse();
                        print_processes(processes, None);
                    } else {
                        print_processes(vec![], Some(entry.uid));
                    }
                }
            }
        }
        Some(("file", sub_matches)) => {
            print_header();
            let file = sub_matches.get_one::<String>("FILE").unwrap();

            // TODO: consider using `std::file::absolute` instead of `canonicalize` when it's stable.
            // Main difference is that `cononicalize` will follow symlinks, which might not always be
            // what the user expects.
            // TODO: Too much nesting!
            if let Ok(target_file) = std::fs::canonicalize(file) {
                for pr in procfs::process::all_processes().unwrap() {
                    if let Ok(p) = pr {
                        if let Ok(fds) = p.fd() {
                            for fdr in fds {
                                if let Ok(fd) = fdr {
                                    match fd.target {
                                        t => match t {
                                            FDTarget::Path(process_file_path) => {
                                                if target_file == process_file_path {
                                                    let mut processes = get_process_parents(
                                                        p.pid,
                                                        &inode_map,
                                                &process_map,
                                                    );
                                                    processes.reverse();
                                                    print_processes(processes, None);
                                                }
                                            }
                                            _ => (),
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Some((&_, _)) => todo!(),
        None => todo!(),
    }
}

// Should be called once before print_processes
fn print_header() {
    println!("{:<8} {:<8} {:<26} {:<26}", "PID", "UID", "EXE", "CMD");
}

/**
Prints found processes OR a single row with all fields except uid set to unknown indicator.
*/
fn print_processes(processes: Vec<ProcessInfo>, uid: Option<u32>) {
    if let Some(uid_n) = uid {
        println!(
            "{:<8} {:<8} {:<26} {:<26}",
            UNKNOWN_INDICATOR, uid_n, UNKNOWN_INDICATOR, UNKNOWN_INDICATOR
        );
    } else {
        for process in processes {
            println!(
                "{:<8} {:<8} {:<26} {:<26}",
                process.pid,
                process
                    .uid
                    .map_or(String::from(UNKNOWN_INDICATOR), |v| format!("{}", v)),
                process.exe,
                process.cmd
            );
        }
    }
}

fn get_process_parents(
    pid: i32,
    inode_map: &HashMap<u64, Stat>,
    process_map: &HashMap<i32, ProcessInfo>,
) -> Vec<ProcessInfo> {
    if let Some(process) = process_map.get(&pid) {
        let mut parents = get_process_parents(process.parent_pid, inode_map, process_map);
        // FIXME: these .to_owned() feels silly...
        parents.push(ProcessInfo {
            pid: process.pid.to_owned(),
            uid: process.uid.to_owned(),
            cmd: process.cmd.to_owned(),
            exe: process.exe.to_owned(),
            parent_pid: process.parent_pid,
        });
        return parents;
    }

    return vec![];
}

fn get_inode_process_parents(
    inode: u64,
    inode_map: &HashMap<u64, Stat>,
    process_map: &HashMap<i32, ProcessInfo>,
) -> Vec<ProcessInfo> {
    if let Some(stat) = inode_map.get(&inode) {
        return get_process_parents(stat.pid, inode_map, process_map);
    }
    return vec![];
}
