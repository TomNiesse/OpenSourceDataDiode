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

    threads.push(thread::spawn(move||{
        loop {
            reader.receive_data().chain_err(|| "Failed to read socket data").chain_unwrap();
        }
    }));

    // TODO: implement fuse things

    // Wait for all threads to end before shutting down
    for thread in threads {
        let _ = thread.join();
    }
}
