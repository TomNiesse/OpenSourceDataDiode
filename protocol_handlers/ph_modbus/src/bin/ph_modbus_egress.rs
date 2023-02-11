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
use ph_modbus::errors::*;
use ph_modbus::*;
use socket_utils::buffered_socket_reader::BufferedSocketReader;
use spsc_bip_buffer::bip_buffer_with_len;
use std::thread;
use structopt::StructOpt;

#[path = "./modbus/modbus_client.rs"]
mod modbus_client;
use modbus_client::{ModbusClient, TcpClient, Frame, Utils};
#[path = "./modbus/modbus_server.rs"]
mod modbus_server;
use modbus_server::{RequestHandler, ModbusDatabank, ModbusServer, DataType, DataPacket};
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
    thread::spawn(move||{
        loop {
            reader.receive_data().chain_err(|| "Failed to read socket data").chain_unwrap();
        }
    });

    match opt.modbus_mode.as_str() {
        "write" => {
            // Create a modbus client
            let tcp_client = TcpClient::new(opt.modbus_address.to_string(), opt.modbus_port, opt.modbus_tcp_timeout, false);
            let mut modbus_client = ModbusClient {
                host: opt.modbus_address,
                port: opt.modbus_port,
                tcp_client,
            };
            let mut latest_packet_id: u16 = 0;
            // Forward the data to the right modbus device
            loop {
                // Read from buffer
                let mut buffer = [0u8;300];
                let _bip_reader = bip_reader_guard.clone();
                let length = read_from_bip_buffer(&mut _bip_reader.lock().unwrap(), &mut buffer);
                let data = &buffer[..length];
                if length > 0 {
                    // Create DataPacket from data
                    let data_packet = DataPacket::from_bytes(data.to_vec());
                    if data_packet.get_id() != latest_packet_id {
                        if data_packet.datatype == DataType::ModbusCommand {
                            // Create a modbus frame
                            let modbus_frame: Frame = Frame::from_bytes(data_packet.get_value().to_vec());
                            if modbus_frame.is_valid() {
                                // Forward the data
                                print!("Sending to Modbus device: ");
                                modbus_frame.print();
                                modbus_client.send_raw(modbus_frame.to_bytes());
                            } else {
                                log::info!("Not handling invalid Modbus frame:");
                            }
                        } else {
                            log::info!("Ignoring data: not a modbus command");
                        }
                        latest_packet_id = data_packet.get_id();
                    }
                }
            }
        },
        "read" => {
            // Create a shared Modbus databank
            let modbus_databank = ModbusDatabank::new();
            let _modbus_databank = modbus_databank.clone();
            // Create a modbus handler and server
            let modbus_request_handler = RequestHandler::new(modbus_databank);
            let mut modbus_server = ModbusServer {
                host: format!("{}:{}", opt.modbus_address, opt.modbus_port),
                handler: modbus_request_handler,
                response_delay: opt.modbus_delay_ms,
                write_buffer: None,
            };
            let mut last_coil_packet_id: u16 = 0;
            let mut last_input_packet_id: u16 = 0;
            let mut last_holding_register_packet_id: u16 = 0;
            let mut last_input_register_packet_id: u16 = 0;
            // Wait for incoming data
            let mut _bip_reader = bip_reader_guard;
            let receive_thread = thread::spawn(move||{
                log::info!("Accepting incoming data.");
                loop {
                    let mut buffer = [0u8;300];
                    let length = read_from_bip_buffer(&mut _bip_reader.lock().unwrap(), &mut buffer);
                    let data = &buffer[..length];
                    let data_packet = DataPacket::from_bytes(data.to_vec());
                    let mut __modbus_databank = _modbus_databank.clone();
                    match data_packet.datatype {
                        DataType::ModbusCommand => {log::info!("Ignoring Modbus command, we're in read mode.");},
                        DataType::CoilValue => {
                            if data_packet.get_id() != last_coil_packet_id {
                                let address = Utils::u8_to_u16(data_packet.value[0], data_packet.value[1]);
                                let status: bool = data_packet.value[2] != 0;
                                __modbus_databank.lock().unwrap().write_coils(
                                    address, [status].to_vec()
                                );
                                last_coil_packet_id = data_packet.get_id();
                            }
                        },
                        DataType::InputValue => {
                            if data_packet.get_id() != last_input_packet_id {
                                let address = Utils::u8_to_u16(data_packet.value[0], data_packet.value[1]);
                                let status: bool = data_packet.value[2] != 0;
                                __modbus_databank.lock().unwrap().write_inputs(
                                    address, [status].to_vec()
                                );
                                last_input_packet_id = data_packet.get_id();
                            }
                        },
                        DataType::HoldingRegisterValue => {
                            if data_packet.get_id() != last_holding_register_packet_id {
                                let address = Utils::u8_to_u16(data_packet.value[0], data_packet.value[1]);
                                let value = Utils::u8_to_u16(data_packet.value[2], data_packet.value[3]);
                                __modbus_databank.lock().unwrap().write_holding_registers(
                                    address, [value].to_vec()
                                );
                                last_holding_register_packet_id = data_packet.get_id();
                            }
                        },
                        DataType::InputRegisterValue => {
                            if data_packet.get_id() != last_input_register_packet_id {
                                let address = Utils::u8_to_u16(data_packet.value[0], data_packet.value[1]);
                                let value = Utils::u8_to_u16(data_packet.value[2], data_packet.value[3]);
                                __modbus_databank.lock().unwrap().write_input_registers(
                                    address, [value].to_vec()
                                );
                                last_input_register_packet_id = data_packet.get_id();
                            }
                        },
                        _ => {log::info!("Ignoring wrong type of data packet")}
                    }
                }
            });

            // Run the modbus server
            modbus_server.run();
            let _ = receive_thread.join();
        },
        _ => {
            unreachable!();
        }
    }

    panic!("Modbus egress proxy stopped working.");
}
