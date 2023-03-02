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

use bip_utils::write_to_bip_buffer;
use logging::*;
use ph_fuse::*;
use spsc_bip_buffer::bip_buffer_with_len;
use std::thread;
use structopt::StructOpt;
use rand::Rng;

use std::sync::{Arc, Mutex};
use socket_utils::socket_writer::SocketWriter;
use std::time::Duration;
use std::thread::JoinHandle;
use std::str::FromStr;
use bip_utils::read_from_bip_buffer;
use spsc_bip_buffer::BipBufferWriter;

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

    // Create a shared bip buffer
    let (bip_writer, bip_reader) = bip_buffer_with_len(MAX_BIP_BUFFER_MESSAGE_SIZE * opt.bip_buffer_element_count as usize);
    let bip_reader_guard = Arc::new(Mutex::new(bip_reader));
    let bip_writer_guard = Arc::new(Mutex::new(bip_writer));

    let mut threads: Vec<JoinHandle<()>> = vec![];

    // Start a thread that forwards data to the other side of the data diode
    let fec_resend_count = opt.fec_resend_count;
    threads.push(thread::spawn(move || {
        let mut _bip_reader = bip_reader_guard.clone();
        let mut socket_writer = SocketWriter::start_listening(&opt.socket_path).expect("Failed to create socket_writer");
        loop {
            let mut buffer = vec![0u8; MAX_BIP_BUFFER_MESSAGE_SIZE];
            let length = read_from_bip_buffer(&mut _bip_reader.lock().unwrap(), &mut buffer);
            if length > 0 {
                for _ in 0..fec_resend_count+1 {
                    socket_writer.send_data(&mut buffer[..length]).expect("Failed to send data");
                }
            }
        }
    }));

    // TODO: implement fuse things

    // Wait for all threads to end before shutting down
    for thread in threads {
        let _ = thread.join();
    }
}