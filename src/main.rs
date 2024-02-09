use std::{collections::HashMap, env};

use procfs::{process::{Stat, FDTarget, Process}, net::TcpState, ProcResult};

struct ProcessInfo {
    cmd: String,
    exe: String,
    parent_pid: i32,
    uid: Option<u32>,
}

const unknown_indicator: &str = "?";

fn main() {
    let args: Vec<_> = env::args().collect();
    let target_port = u16::from_str_radix(&args[1], 10).unwrap();

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
            let mut exe = String::from(unknown_indicator);
            if let Ok(str) = process.exe() {
                exe = str.to_str().unwrap().to_string();
            }
            process_map.insert(process.pid, ProcessInfo{
                cmd: process.cmdline().unwrap().join(" "),
                exe: exe,
                parent_pid: ppid,
                uid: process.uid().map_or(None, |v| Some(v)),
            });
        }
    }

    println!("{:<8} {:<8} {:<26} {:<26}", "PID", "UID", "EXE", "CMD");

    let mut is_first = true;

    // get the tcp table
    let tcp = procfs::net::tcp().unwrap();
    let tcp6 = procfs::net::tcp6().unwrap();
    for entry in tcp.into_iter().chain(tcp6) {
        if entry.local_address.port() == target_port && entry.state == TcpState::Listen {
            if !is_first {
                println!();
            }
            is_first = false;

            if let Some(stat) = inode_map.get(&entry.inode) {
                let /* mut */ pid = &stat.pid;
                let mut processes = get_process_parents(*pid, &inode_map, &process_map);
                processes.reverse();
                for process in processes {
                    
                    println!("{:<8} {:<8} {:<26} {:<26}", pid, process.uid.map_or(String::from("?"), |v| format!("{}", v)), process.exe, process.cmd);
                }
            } else {
                println!("{:<8} {:<8} {:<26} {:<26}", unknown_indicator, entry.uid, unknown_indicator, unknown_indicator);
            }
        }
    }
}

fn get_process_parents(pid: i32, inode_map: &HashMap<u64, Stat>, process_map: &HashMap<i32, ProcessInfo>) -> Vec<ProcessInfo> {
    if let Some(process) = process_map.get(&pid) {
        let mut parents = get_process_parents(process.parent_pid, inode_map, process_map);
        // FIXME: these .to_owned() feels silly...
        parents.push(ProcessInfo { uid: process.uid.to_owned(), cmd: process.cmd.to_owned(), exe: process.exe.to_owned(), parent_pid: process.parent_pid });
        return parents;
    }

    return vec![];
}

fn get_inode_process_parents(inode: u64, inode_map: &HashMap<u64, Stat>, process_map: &HashMap<i32, ProcessInfo>) -> Vec<ProcessInfo> {
    if let Some(stat) = inode_map.get(&inode) {
        return get_process_parents(stat.pid, inode_map, process_map)
    }
    return vec![];
}
