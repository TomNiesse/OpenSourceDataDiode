// Copyright 2020 Ministerie van Defensie
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bip_utils::read_from_bip_buffer;
use logging::*;
use ph_fuse::errors::*;
use ph_fuse::*;
use socket_utils::buffered_socket_reader::BufferedSocketReader;
use spsc_bip_buffer::bip_buffer_with_len;
use std::thread;
use std::thread::JoinHandle;
use structopt::StructOpt;
use std::sync::{Arc, Mutex};
use std::ffi::OsStr;
use std::time::Duration;

#[path = "./fuse/diode_fs.rs"]
mod diode_fs;
use diode_fs::{DiodeFS, FilesystemCommit};

#[path = "./fuse/diode_fs_injector.rs"]
mod diode_fs_injector;
use diode_fs_injector::DiodeFSInjector;

fn main() {
    let opt = arguments::OptIngress::from_args();
    set_syslog(
        opt.from_host_sys_log.as_str(),
        opt.from_port_sys_log.to_string().as_str(),
        opt.to_host_sys_log.as_str(),
        opt.to_port_sys_log.to_string().as_str(),
        opt.log_level.as_str(),
        opt.handler_name.as_str(),
    ).expect("Could not set syslog");

    // Create a shared buffer
    let (bip_writer, bip_reader) = bip_buffer_with_len(MAX_BIP_BUFFER_MESSAGE_SIZE * opt.bip_buffer_element_count as usize);
    // Store any incoming data into the shared buffer
    let bip_reader_guard = Arc::new(Mutex::new(bip_reader));
    let mut reader = BufferedSocketReader::new(&opt.socket_path, bip_writer).expect("Failed to create socket_reader");

    let mut threads: Vec<JoinHandle<()>> = vec![];

    // Start a thread that reads incoming data and stores it in the bip buffer
    threads.push(thread::spawn(move||{
        loop {
            reader.receive_data().chain_err(|| "Failed to read socket data").chain_unwrap();
        }
    }));

    // Create a DiodeFS object
    let diode_fs = DiodeFS::new();
    // Make sure the filesystem and commits can be accessed from the outside
    let filesystem_items = diode_fs.get_filesystem_items().clone();
    let filesystem_commits = diode_fs.get_filesystem_commits().clone();

    // Start a thread that mounts a fuse filesystem
    threads.push(thread::spawn(move||{
        let mountpoint = "/fusemount".to_string();   // TODO: MAKE THIS A CONFIG OPTION IF POSSIBLE
        let options = [OsStr::new("-o"), OsStr::new("atomic_o_trunc")];
        println!("Starting to mount filesystem");
        let _ = fuse::mount(diode_fs, mountpoint, &options);
    }));

    // Start a thread that injects filesystem items based on incoming commits
    threads.push(thread::spawn(move||{
        // Create an injector
        let mut injector = DiodeFSInjector {
            filesystem_items,
            filesystem_commits
        };
        // Inject incoming commits into the filesystem
        loop {
            let mut buffer = [0u8;65535];
            let length = read_from_bip_buffer(&mut bip_reader_guard.lock().unwrap(), &mut buffer);
            let data = &buffer[..length];
            println!("Received data: {:?}", data);
            let commit = FilesystemCommit::from_bytes(data.to_vec());
            injector.handle_commit(commit);
            // Sleep for a few milleseonds to allow the FS thread to do stuff
            thread::sleep(Duration::from_millis(10)); // TODO: MAKE THIS A CONFIG OPTION
        }
    }));

    // Wait for all threads to end before shutting down
    for thread in threads {
        let _ = thread.join();
    }
}
