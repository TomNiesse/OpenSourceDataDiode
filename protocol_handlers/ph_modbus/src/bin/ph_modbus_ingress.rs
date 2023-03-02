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
use ph_modbus::*;
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

#[path = "./modbus/modbus_server.rs"]
mod modbus_server;
use modbus_server::{RequestHandler, ModbusDatabank, ModbusServer, DataType, DataPacket};

#[path = "./modbus/modbus_client.rs"]
mod modbus_client;
use modbus_client::{ModbusClient, TcpClient};

#[path = "./modbus/utils.rs"]
mod utils;
use utils::{Utils};

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
    let forwarding_thread = thread::spawn(move || {
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
    });
    threads.push(forwarding_thread);

    // Start threads that read from or write to a Modbus device
    match opt.modbus_mode.as_str() {
        "write" => {
            // Create a modbus server
            let modbus_databank = ModbusDatabank::new();
            let request_handler = RequestHandler::new(modbus_databank);
            let mut modbus_server = ModbusServer {
                handler: request_handler,
                host: format!("{}:{}", opt.modbus_address, opt.modbus_port),
                write_buffer: Some(bip_writer_guard),
                response_delay: opt.modbus_delay_ms,
            };
            // Run the modbus server
            modbus_server.run();
        },
        "read" => {
            // Start a thread in which coils are read
            let host = opt.modbus_address.clone();
            let mut _bip_writer = bip_writer_guard.clone();
            let coil_read_thread = thread::spawn(move||{
                read_coils(&opt.modbus_coil_addresses_to_read, _bip_writer, &host, opt.modbus_port, opt.modbus_tcp_timeout, opt.modbus_delay_ms);
            });
            threads.push(coil_read_thread);

            // Start a thread in which inputs are read
            thread::sleep(Duration::from_millis(opt.modbus_delay_ms/4));
            let host = opt.modbus_address.clone();
            let mut _bip_writer = bip_writer_guard.clone();
            let input_read_thread = thread::spawn(move||{
                read_inputs(&opt.modbus_input_addresses_to_read, _bip_writer, &host, opt.modbus_port, opt.modbus_tcp_timeout, opt.modbus_delay_ms);
            });
            threads.push(input_read_thread);

            // Start a thread in which holding registers are read
            thread::sleep(Duration::from_millis(opt.modbus_delay_ms/4));
            let host = opt.modbus_address.clone();
            let mut _bip_writer = bip_writer_guard.clone();
            let holding_register_read_thread = thread::spawn(move||{
                read_holding_registers(&opt.modbus_holding_register_addresses_to_read, _bip_writer, &host, opt.modbus_port, opt.modbus_tcp_timeout, opt.modbus_delay_ms);
            });
            threads.push(holding_register_read_thread);

            // Start a thread in which input registers are read
            thread::sleep(Duration::from_millis(opt.modbus_delay_ms/4));
            let host = opt.modbus_address.clone();
            let mut _bip_writer = bip_writer_guard;
            let input_register_read_thread = thread::spawn(move||{
                read_input_registers(&opt.modbus_input_register_addresses_to_read, _bip_writer, &host, opt.modbus_port, opt.modbus_tcp_timeout, opt.modbus_delay_ms);
            });
            threads.push(input_register_read_thread);
        }
        _ => {
            unreachable!(); // User should not reach this
        }
    }
    // Wait for all threads to end before shutting down
    for thread in threads {
        let _ = thread.join();
    }
}

fn read_coils(modbus_coil_addresses_to_read: &String, bip_writer: Arc<Mutex<BipBufferWriter>>, modbus_address: &String, modbus_port: u16, modbus_tcp_timeout: u64, modbus_delay_ms: u64) {
    // Read the given coil addresses and send them over
    let tcp_client = TcpClient::new(modbus_address.to_string(), modbus_port, modbus_tcp_timeout, true);
    let mut modbus_client = ModbusClient {
        host: modbus_address.to_string(),
        port: modbus_port,
        tcp_client
    };
    if !modbus_coil_addresses_to_read.is_empty() {
        loop {
            for item in modbus_coil_addresses_to_read.split(',') {
                let response: Vec<bool>;
                let start: u16;

                // Check if a range is requested
                if item.contains('-') {
                    // Read range of data from Modbus device
                    let (_start, _end) = item.split_once('-').unwrap();
                    start = u16::from_str(_start).unwrap();
                    let end = u16::from_str(_end).unwrap();
                    response = modbus_client.read_coils(start, end-start+1);
                } else {
                    start = u16::from_str(item).unwrap();
                    response = modbus_client.read_coils(start, 1);
                }

                // Forward all the data
                if !response.is_empty() {
                    for (index, status) in response.iter().enumerate() {
                        let data_packet = DataPacket {
                            datatype: DataType::CoilValue,
                            id: rand::thread_rng().gen_range(0..65535) as u16,
                            value: [
                            Utils::u16_to_u8(start+index as u16)[0],    // address
                            Utils::u16_to_u8(start+index as u16)[1],    // address
                            *status as u8                               // coil status
                            ].to_vec()
                        };
                        write_to_bip_buffer(&mut bip_writer.lock().unwrap(), &data_packet.to_bytes());
                    }
                } else {
                    log::info!("Could not read coil value(s) for {:?}", item);
                }
                if modbus_delay_ms > 0 {
                    thread::sleep(Duration::from_millis(modbus_delay_ms));
                }
            }
        }
    } else {
        log::info!("Not reading any coils: No coils were given.");
    }
}

fn read_inputs(modbus_input_addresses_to_read: &String, bip_writer: Arc<Mutex<BipBufferWriter>>, modbus_address: &String, modbus_port: u16, modbus_tcp_timeout: u64, modbus_delay_ms: u64) {
    // Read the given input addresses and send them over
    let tcp_client = TcpClient::new(modbus_address.to_string(), modbus_port, modbus_tcp_timeout, true);
    let mut modbus_client = ModbusClient {
        host: modbus_address.to_string(),
        port: modbus_port,
        tcp_client
    };
    if !modbus_input_addresses_to_read.is_empty() {
        loop {
            for item in modbus_input_addresses_to_read.split(',') {
                let response: Vec<bool>;
                let start: u16;

                // Check if a range is requested
                if item.contains('-') {
                    // Read range of data from Modbus device
                    let (_start, _end) = item.split_once('-').unwrap();
                    start = u16::from_str(_start).unwrap();
                    let end = u16::from_str(_end).unwrap();
                    response = modbus_client.read_discrete_inputs(start, end-start+1);
                } else {
                    start = u16::from_str(item).unwrap();
                    response = modbus_client.read_discrete_inputs(start, 1);
                }

                // Forward all the data
                if !response.is_empty() {
                    for (index, status) in response.iter().enumerate() {
                        let data_packet = DataPacket {
                            datatype: DataType::InputValue,
                            id: rand::thread_rng().gen_range(0..65535) as u16,
                            value: [
                                Utils::u16_to_u8(start+index as u16)[0],    // address
                                Utils::u16_to_u8(start+index as u16)[1],    // address
                                *status as u8                               // input status
                            ].to_vec()
                        };
                        write_to_bip_buffer(&mut bip_writer.lock().unwrap(), &data_packet.to_bytes());
                    }
                } else {
                    log::info!("Could not read input value(s) for {:?}", item);
                }
                if modbus_delay_ms > 0 {
                    thread::sleep(Duration::from_millis(modbus_delay_ms));
                }
            }
        }
    } else {
        log::info!("Not reading any inputs: No inputs were given.");
    }
}

fn read_holding_registers(modbus_holding_register_addresses_to_read: &String, bip_writer: Arc<Mutex<BipBufferWriter>>, modbus_address: &String, modbus_port: u16, modbus_tcp_timeout: u64, modbus_delay_ms: u64) {
    // Read the given input addresses and send them over
    let tcp_client = TcpClient::new(modbus_address.to_string(), modbus_port, modbus_tcp_timeout, true);
    let mut modbus_client = ModbusClient {
        host: modbus_address.to_string(),
        port: modbus_port,
        tcp_client
    };
    if !modbus_holding_register_addresses_to_read.is_empty() {
        loop {
            for item in modbus_holding_register_addresses_to_read.split(',') {
                let response: Vec<u16>;
                let start: u16;

                // Check if a range is requested
                if item.contains('-') {
                    // Read range of data from Modbus device
                    let (_start, _end) = item.split_once('-').unwrap();
                    start = u16::from_str(_start).unwrap();
                    let end = u16::from_str(_end).unwrap();
                    response = modbus_client.read_holding_registers(start, end-start+1);
                } else {
                    start = u16::from_str(item).unwrap();
                    response = modbus_client.read_holding_registers(start, 1);
                }

                // Forward all the data
                if !response.is_empty() {
                    for (index, status) in response.iter().enumerate() {
                        let data_packet = DataPacket {
                            datatype: DataType::HoldingRegisterValue,
                            id: rand::thread_rng().gen_range(0..65535) as u16,
                            value: [
                                Utils::u16_to_u8(start+index as u16)[0],    // address
                                Utils::u16_to_u8(start+index as u16)[1],    // address
                                Utils::u16_to_u8(*status)[0],               // register status
                                Utils::u16_to_u8(*status)[1]                // register status
                            ].to_vec()
                        };
                        write_to_bip_buffer(&mut bip_writer.lock().unwrap(), &data_packet.to_bytes());
                    }
                } else {
                    log::info!("Could not read holding register value(s) for {:?}", item);
                }
                if modbus_delay_ms > 0 {
                    thread::sleep(Duration::from_millis(modbus_delay_ms));
                }
            }
        }
    } else {
        log::info!("Not reading any holding registers: No holding registers were given.");
    }
}

fn read_input_registers(modbus_input_register_addresses_to_read: &String, bip_writer: Arc<Mutex<BipBufferWriter>>, modbus_address: &String, modbus_port: u16, modbus_tcp_timeout: u64, modbus_delay_ms: u64) {
    // Read the given input addresses and send them over
    let tcp_client = TcpClient::new(modbus_address.to_string(), modbus_port, modbus_tcp_timeout, true);
    let mut modbus_client = ModbusClient {
        host: modbus_address.to_string(),
        port: modbus_port,
        tcp_client
    };
    if !modbus_input_register_addresses_to_read.is_empty() {
        loop {
            for item in modbus_input_register_addresses_to_read.split(',') {
                let response: Vec<u16>;
                let start: u16;

                // Check if a range is requested
                if item.contains('-') {
                    // Read range of data from Modbus device
                    let (_start, _end) = item.split_once('-').unwrap();
                    start = u16::from_str(_start).unwrap();
                    let end = u16::from_str(_end).unwrap();
                    response = modbus_client.read_input_registers(start, end-start+1);
                } else {
                    start = u16::from_str(item).unwrap();
                    response = modbus_client.read_input_registers(start, 1);
                }

                // Forward all the data
                if !response.is_empty() {
                    for (index, status) in response.iter().enumerate() {
                        let data_packet = DataPacket {
                            datatype: DataType::InputRegisterValue,
                            id: rand::thread_rng().gen_range(0..65535) as u16,
                            value: [
                                Utils::u16_to_u8(start+index as u16)[0],    // address
                                Utils::u16_to_u8(start+index as u16)[1],    // address
                                Utils::u16_to_u8(*status)[0],               // register status
                                Utils::u16_to_u8(*status)[1]                // register status
                            ].to_vec()
                        };
                        write_to_bip_buffer(&mut bip_writer.lock().unwrap(), &data_packet.to_bytes());
                    }
                } else {
                    log::info!("Could not read input register value(s) for {:?}", item);
                }
                if modbus_delay_ms > 0 {
                    thread::sleep(Duration::from_millis(modbus_delay_ms));
                }
            }
        }
    } else {
        log::info!("Not reading any input registers: No input registers were given.");
    }
}
