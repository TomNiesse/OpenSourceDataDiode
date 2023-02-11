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

use structopt::StructOpt;
///Commandline arguments used to run ph_modbus_ingress.
#[derive(StructOpt)]
pub struct OptIngress {
    #[structopt(
        long = "socket_path",
        default_value = "/tmp/handler_to_transport",
        help = "Location of the socket"
    )]
    pub socket_path: String,

    ///The maximum amount of elements the bip buffer can store.
    ///The size of a single element is 1Mb.
    #[structopt(long = "bip_buffer_element_count", default_value = "10")]
    pub bip_buffer_element_count: usize,

    ///Port the stats handler is listening on.
    #[structopt(long = "listening_port", default_value = "1235")]
    pub listening_port: u16,

    ///StatsD server host.
    #[structopt(long = "stats_server_address", default_value = "127.0.0.1")]
    pub host_stats_server: String,

    ///StatsD server port.
    #[structopt(long = "stats_server_port", default_value = "8125")]
    pub port_stats_server: u16,

    ///From syslog server host
    #[structopt(long = "from_host_sys_log", default_value = "0.0.0.0")]
    pub from_host_sys_log: String,

    ///From syslog server port
    #[structopt(long = "from_port_sys_log", default_value = "8127")]
    pub from_port_sys_log: u16,

    ///To syslog udp host
    #[structopt(long = "to_host_sys_log", default_value = "127.0.0.1")]
    pub to_host_sys_log: String,

    ///To syslog udp port
    #[structopt(long = "to_port_sys_log", default_value = "8082")]
    pub to_port_sys_log: u16,

    ///Log level for logging
    #[structopt(long = "log_level", default_value = "Warn")]
    pub log_level: String,

    ///Log level for logging
    #[structopt(long = "handler_name", default_value = "ph_modbus_ingress")]
    pub handler_name: String,

    // The proxy mode for the Modbus protocol handler.
    // Read mode: Read configured registers and send them to the egress proxy (poll a Modbus device and send data)
    // Write mode: Wait for incoming Modbus commands and send them to the egress proxy (wait for instructions and send data)
    #[structopt(long = "modbus_mode", default_value = "write")]
    pub modbus_mode: String,

    // Modbus address to read from.
    // Write mode: Read from a Modbus master and forward the data to right Modbus device
    // Read mode:  Read from the given Modbus device and forward the data to the egress proxy
    #[structopt(long = "modbus_address", default_value = "0.0.0.0")]
    pub modbus_address: String,

    // Modbus port to read from.
    // Write mode: Receive data from a Modbus master on the given port and forward the data to the egress proxy
    // Read mode:  Read from the given Modbus device port and forward the data to the egress proxy
    #[structopt(long = "modbus_port", default_value = "502")]
    pub modbus_port: u16,

    // Write mode: Responses are delayed by the given amount of ms to prevent Modbus masters from writing data too fast.
    //             The virtual Modbus server is way faster than any real world device, causing a potential backlog at the egress proxy.
    //             This option slows it down by the given amount of milliseconds.
    // Read mode:  Requests have a delay between them, to reduce the amount of requests per second a Modbus device gets.
    //             If a device has multiple/many masters, consider setting this option to something higher than 0.
    #[structopt(long = "modbus_delay_ms", default_value = "0")]
    pub modbus_delay_ms: u64,

    // Read mode: Read the given coil addresses and send them to the egress proxy
    // Write mode: This option does nothing
    #[structopt(long = "modbus_coil_addresses_to_read", default_value = "")]
    pub modbus_coil_addresses_to_read: String,  // This cannot be a Vec<u16>, this string will be split into values using str.split(',') later

    // Read mode: Read the given input addresses and send them to the egress proxy
    // Write mode: This option does nothing
    #[structopt(long = "modbus_input_addresses_to_read", default_value = "")]
    pub modbus_input_addresses_to_read: String,

    // Read mode: Read the given holding register addresses and send them to the egress proxy
    // Write mode: This option does nothing
    #[structopt(long = "modbus_holding_register_addresses_to_read", default_value = "")]
    pub modbus_holding_register_addresses_to_read: String,

    // Read mode: Read the given input register addresses and send them to the egress proxy
    // Write mode: This option does nothing
    #[structopt(long = "modbus_input_register_addresses_to_read", default_value = "")]
    pub modbus_input_register_addresses_to_read: String,

    // How long should a TCP connection attempt take? (in milliseconds)
    #[structopt(long = "modbus_tcp_timeout", default_value = "250")]
    pub modbus_tcp_timeout: u64,

    // How many extra times should a message be sent (forward error correction)?
    // If this value is set too low (especially in write mode), data loss may occur.
    // If this value is set too high (especially in read mode), a high latency may occur.
    #[structopt(long = "fec_resend_count", default_value = "1")]
    pub fec_resend_count: u8,
}

// Commandline arguments used to run ph_modbus_egress.
#[derive(StructOpt)]
pub struct OptEgress {
    #[structopt(
        long = "socket_path",
        default_value = "/tmp/transport_to_handler",
        help = "Location of the socket"
    )]
    pub socket_path: String,

    ///Port the stats handler is listening on.
    #[structopt(long = "listening_port", default_value = "1235")]
    pub listening_port: u16,

    ///StatsD server host.
    #[structopt(long = "stats_server_address", default_value = "127.0.0.1")]
    pub host_stats_server: String,

    ///StatsD server port.
    #[structopt(long = "stats_server_port", default_value = "8125")]
    pub port_stats_server: u16,

    ///From syslog server host
    #[structopt(long = "from_host_sys_log", default_value = "0.0.0.0")]
    pub from_host_sys_log: String,

    ///From syslog server port
    #[structopt(long = "from_port_sys_log", default_value = "8127")]
    pub from_port_sys_log: u16,

    ///To syslog udp host
    #[structopt(long = "to_host_sys_log", default_value = "127.0.0.1")]
    pub to_host_sys_log: String,

    ///To syslog udp port
    #[structopt(long = "to_port_sys_log", default_value = "8082")]
    pub to_port_sys_log: u16,

    ///The maximum amount of elements the bip buffer can store.
    ///The size of a single element is 1Mb.
    #[structopt(long = "bip_buffer_element_count", default_value = "100")]
    pub bip_buffer_element_count: usize,

    ///Log level for logging
    #[structopt(long = "log_level", default_value = "Warn")]
    pub log_level: String,

    ///Log level for logging
    #[structopt(long = "handler_name", default_value = "ph_modbus_egress")]
    pub handler_name: String,

    // The proxy mode for the Modbus protocol handler.
    // Read mode: Receive given data and store it in a Modbus database. Handle requests using a virtual Modbus server.
    // Write mode: Receive a Modbus command, check it's validity and forward it to the given Modbus address and port
    #[structopt(long = "modbus_mode", default_value = "write")]
    pub modbus_mode: String,

    // Modbus address.
    // Write mode: Send all Modbus commands to this Modbus device address
    // Read mode: Listen for connections on this address
    #[structopt(long = "modbus_address", default_value = "0.0.0.0")]
    pub modbus_address: String,

    // Modbus port.
    // Write mode: Send all Modbus commands to this Modbus device port
    // Read mode: Listen for connections on this port
    #[structopt(long = "modbus_port", default_value = "502")]
    pub modbus_port: u16,

    // How long should a TCP connection attempt take? (in milliseconds)
    #[structopt(long = "modbus_tcp_timeout", default_value = "250")]
    pub modbus_tcp_timeout: u64,
}
