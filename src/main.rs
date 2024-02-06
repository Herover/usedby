use std::{collections::HashMap, env};

use procfs::{process::{Process, Stat, FDTarget}, net::TcpState};

fn main() {
    let args: Vec<_> = env::args().collect();
    let target_port = u16::from_str_radix(&args[1], 10).unwrap();

    let all_procs = procfs::process::all_processes().unwrap();
    
    // build up a map between socket inodes and process stat info:
    let mut inode_map: HashMap<u64, Stat> = HashMap::new();
    let mut process_map: HashMap<i32, (String, i32, String)> = HashMap::new();
    for p in all_procs {
        let process = p.unwrap();
        let mut ppid = 0;
        if let (Ok(stat), Ok(fds)) = (process.stat(), process.fd()) {
            ppid = stat.ppid;
            for fd in fds {
                match fd.unwrap().target {
                    FDTarget::Socket(inode) => inode_map.insert(inode, stat.clone()),
                    FDTarget::Path(inode) => None,
                    e => None,
                };
            }
        }
        let mut exe = String::from("?");
        if let Ok(str) = process.exe() {
            exe = str.to_str().unwrap().to_string();
        }
        process_map.insert(process.pid, (process.cmdline().unwrap().join(" "), ppid, exe));
    }

    println!("{:<8} {:<26} {:<26}", "PID", "EXE", "CMD");

    // get the tcp table
    let tcp = procfs::net::tcp().unwrap();
    let tcp6 = procfs::net::tcp6().unwrap();
    for entry in tcp.into_iter().chain(tcp6) {
        // find the process (if any) that has an open FD to this entry's inode
        // let local_address = format!("{}", entry.local_address);
        // let remote_addr = format!("{}", entry.remote_address);
        // let state = format!("{:?}", entry.state);
        
        if entry.local_address.port() == target_port && entry.state == TcpState::Listen {
            if let Some(stat) = inode_map.get(&entry.inode) {
                let mut pid = &stat.pid;
                loop {
                    if pid == &0 {
                        break;
                    }
                    if let Some(process) = process_map.get(pid) {
                        println!("{:<8} {:<26} {:<26}", pid, process.2, process.0);
                        pid = &process.1;
                    } else {
                        pid = &0;
                    }
                }
            } else {
                
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
