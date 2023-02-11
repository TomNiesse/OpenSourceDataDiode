use std::net::{TcpStream, Shutdown};
use std::io::{Read, Write};
use std::time::Duration;
use std::io::Error;

pub struct TcpClient {
    host: String,
    stream: Result<TcpStream, Error>,
    tcp_timeout: u64,
    stay_connected: bool
}

#[allow(dead_code)]
impl TcpClient {
    pub fn new(host: String, port: u16, tcp_timeout: u64, stay_connected: bool) -> Self {
        let _host = format!("{}:{}", host, port);
        log::info!("Connecting to host {:?}", _host);
        Self {
            stream: TcpClient::connect(_host.clone()),
            host: _host,
            stay_connected,
            tcp_timeout
        }
    }
    fn connect(host: String) -> Result<TcpStream, Error> {
        let mut _stream;
        loop {
            match TcpStream::connect(host.clone()) {
                Ok(stream) => {
                    _stream = Ok(stream);
                    log::info!("Connected!");
                    break;
                },
                Err(_) => {
                    log::info!("Unable to connect to Modbus device");
                }
            }
        }
        _stream
    }
    fn disconnect(&self) {
        let _ = self.stream.as_ref().unwrap().shutdown(Shutdown::Both);
    }
    fn check_connection(&mut self, retry: bool) -> bool {
        match self.stream {
            Ok(_) => {
                true
            },
            Err(_) => {
                self.stream = TcpStream::connect(self.host.clone());
                if retry {
                    return self.check_connection(false);
                }
                false
            }
        }
    }
    pub fn send(&mut self, host: String, port: u16, mut data: Vec<u8>) -> Vec<u8> {
        if self.check_connection(true) {
            self.stream.as_ref().expect("Stream stopped existing").set_read_timeout(Some(Duration::from_millis(self.tcp_timeout))).expect("Failed to call set_read_timeout");
            self.stream.as_ref().expect("Stream stopped existing").set_write_timeout(Some(Duration::from_millis(self.tcp_timeout))).expect("Failed to call set_write_timeout");
            let buffer: Vec<u8>;
            // write to stream, but check if the cable was removed
            match self.stream.as_ref().expect("Could not write to stream.").write_all(&data) {
                Ok(_) => {
                    // Ignore
                },
                Err(_) => {
                    // Reconnect and retry
                    while !self.check_connection(true) {}
                    self.send(host, port, data.clone());
                }
            }
            data.resize(4096, 0u8);
            match self.stream.as_ref().expect("Could not read response from host").read(&mut data) {
                Ok(size) => {
                    buffer = data[0..size].to_vec();
                    return buffer;
                },
                Err(_) => {
                    log::info!("Unable to read response from Modbus device");
                    while !self.check_connection(true) {}
                }
            };
        } else {
            log::info!("Unable to send data to Modbus device");
            self.stream = TcpStream::connect(self.host.clone());
        }
        if !self.stay_connected {
            self.disconnect();
        }
        [].to_vec()
    }
}
