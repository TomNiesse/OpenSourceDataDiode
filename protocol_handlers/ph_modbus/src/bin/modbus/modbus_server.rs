use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use rand::Rng;

mod request_handler;
pub use request_handler::{RequestHandler, Frame, ModbusDatabank};
mod data_packet;
pub use data_packet::{DataPacket, DataType};
use spsc_bip_buffer::BipBufferWriter;
use bip_utils::write_to_bip_buffer;

pub struct ModbusServer {
    pub host: String,
    pub handler: Arc<Mutex<RequestHandler>>,
    pub write_buffer: Option<Arc<Mutex<BipBufferWriter>>>,
    pub response_delay: u64,
}

#[allow(dead_code)]
impl ModbusServer {
    pub fn run(&mut self) {
        let listener = TcpListener::bind(&self.host).unwrap();

        // Accept connections and process them, spawning a new thread for each one
        log::info!("Server listening on {}", &self.host);
        for stream in listener.incoming() {
            self.handle_request(stream.unwrap());
        }
        drop(listener);
    }
    pub fn handle_request(&mut self, mut stream: TcpStream) {
        let handler = self.handler.clone();
        let buffer = self.write_buffer.clone();
        let delay = self.response_delay;
        thread::spawn(move||{
            let mut data = [0u8; 4096];
            loop {
                if let Ok(size) = stream.read(&mut data) {
                    let mut input_frame = Frame::from_bytes(data[0..size].to_vec());
                    if input_frame.is_valid() {
                        // Ignore any read requests, forward any other (custom) request
                        // (applies to write mode only)
                        match input_frame.pdu().get_function_code() {
                            0x01 | 0x02 | 0x03 | 0x04 | 0x07 | 0x08 | 0x0B | 0x0C | 0x11 | 0x14 | 0x18 | 0x2B => {
                                // Do not forward these requests to the modbus device
                            }
                            _ => {
                                if buffer.is_some() {
                                    let id = rand::thread_rng().gen_range(0..65535) as u16;
                                    // Wrap the request in a DataPacket so the receiver knows what type of data it is
                                    let udp_data = DataPacket::new(DataType::ModbusCommand, input_frame.to_bytes(), id);
                                    // Add the data packet to buffer
                                    write_to_bip_buffer(&mut buffer.as_ref().expect("User somehow managed to write to the egress proxy while in read mode.").lock().unwrap(), &udp_data.to_bytes());
                                }
                            }
                        }
                        // Respond to the query
                        let output_frame: Frame = handler.lock().unwrap().handle(input_frame);
                        if delay > 0 {
                            thread::sleep(Duration::from_millis(delay));
                        }
                        let _ = stream.write_all(&output_frame.to_bytes());
                    } else {
                        log::info!("Received invalid frame:");
                        input_frame.print();
                        let output_frame: Frame = RequestHandler::server_device_failure(input_frame);
                        if delay > 0 {
                            thread::sleep(Duration::from_millis(delay));
                        }
                        let _ = stream.write_all(&output_frame.to_bytes());
                        break;
                    }
                }
            }
        });
    }
}
