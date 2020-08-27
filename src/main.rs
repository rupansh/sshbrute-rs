/*
   Copyright 2020 Rupansh Sekar

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

     http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

mod args;
use args::Args;

use clap::Clap;
use num_cpus;
use ssh2::Session;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, prelude::*},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering}
    },
    net::TcpStream,
};
use threadpool::ThreadPool;


fn main() -> std::io::Result<()> {
    let args: Args = Args::parse();
    let verbose = args.verbose > 0;

    let host_list_f = File::open(&args.hostlist)?;
    let hosts: Vec<String> = 
    BufReader::new(host_list_f)
        .lines()
        .map(|l| l.expect("Could not load hosts"))
        .collect();

    let wordlist_f = File::open(&args.wordlist)?;
    let wordlist: Vec<[String; 2]> = 
    BufReader::new(wordlist_f)
        .lines()
        .map(|l| {
            let res = l.expect("Could not load wordlist");
            if !res.contains(":") {
                panic!("Invalid wordlist")
            }
            let res: Vec<&str> = res.split(":").collect();
            [res[0].to_string(), res[1].to_string()]
        })
        .collect();

    let nt = num_cpus::get();
    let num_threads = if args.threads.is_some() {
        args.threads.unwrap().parse::<usize>().unwrap_or(nt)
    } else { nt };

    let pool = ThreadPool::new(num_threads);
    let host_done = Arc::new(Mutex::new(HashMap::<usize, Arc<AtomicBool>>::new()));

    for (i, host) in hosts.iter().enumerate() {
        let thost_done = Arc::clone(&host_done);
        let mut thost_done = thost_done.lock().unwrap();
        thost_done.insert(i, Arc::new(AtomicBool::new(false)));
        std::mem::drop(thost_done);

        let wordlist = wordlist.clone();
        for combo in wordlist {
            let host_done = host_done.clone();
            let host = host.clone();
            pool.execute(move || {
                let host_done0 = host_done.clone();
                let host_done0 = host_done0.lock().unwrap();
                if host_done0.get(&i).unwrap().load(Ordering::Relaxed) {
                    return;
                }
                std::mem::drop(host_done0);

                let res = auth_ssh(&host, &combo[0], &combo[1]);

                let host_done1 = host_done.clone();
                let host_done1 = host_done1.lock().unwrap();
                let host_status = host_done1.get(&i).unwrap();

                if res.is_ok() {
                    println!("PASSED Host: {}, Combo: {}:{}", &host, &combo[0], &combo[1]);
                    host_status.store(true, Ordering::Relaxed);
                } else if verbose && !host_status.load(Ordering::Relaxed) {
                    println!("FAILED Host: {}, Combo: {}:{}, Cause: {}",  &host, &combo[0], &combo[1], res.err().unwrap());
                }
            })
        }
    }

    pool.join();

    Ok(())
}

fn auth_ssh(ip: &str, username: &str, password: &str) -> Result<(), String> {
    let tcp = match TcpStream::connect(format!("{}:22", ip)) {
        Ok(t) => t,
        Err(e) => return Err(e.to_string())
    };

    let mut session = Session::new().unwrap();
    session.set_tcp_stream(tcp);
    match session.handshake() {
        Err(e) => return Err(e.to_string()),
        _ => {}
    }
    return match session.userauth_password(username, password) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string())
    }
}