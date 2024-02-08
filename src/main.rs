use std::{collections::HashMap, env};

use procfs::{process::{Stat, FDTarget}, net::TcpState, ProcResult};

struct ProcessInfo {
    cmd: String,
    exe: String,
    parent_pid: i32,
    uid: ProcResult<u32>,
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
                uid: process.uid(),
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
                let mut pid = &stat.pid;
                loop {
                    if pid == &0 {
                        break;
                    }
                    if let Some(process) = process_map.get(pid) {
                        println!("{:<8} {:<8} {:<26} {:<26}", pid, process.uid.as_ref().map_or(String::from(unknown_indicator), |v| format!("{}", v)), process.exe, process.cmd);
                        pid = &process.parent_pid;
                    } else {
                        println!("{pid}");
                        pid = &0;
                    }
                }
            } else {
                println!("{:<8} {:<8} {:<26} {:<26}", unknown_indicator, entry.uid, unknown_indicator, unknown_indicator);
            }
        }
        /* if let Some(stat) = map.get(&entry.inode) {
            println!(
                "{:<26} {:<26} {:<15} {:<12} {}/{}",
                local_address, remote_addr, state, entry.inode, stat.pid, stat.comm
            );
        } else {
            // We might not always be able to find the process associated with this socket
            println!(
                "{:<26} {:<26} {:<15} {:<12} -",
                local_address, remote_addr, state, entry.inode
            );
        } */
    }
}
