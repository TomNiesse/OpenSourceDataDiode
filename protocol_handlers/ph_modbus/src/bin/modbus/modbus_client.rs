#[path = "./frame.rs"] mod frame;
pub use frame::{Frame, Mbap, Pdu};
#[path = "./utils.rs"] mod utils;
pub use utils::Utils;
#[path = "./tcp_client.rs"] mod tcp_client;
pub use tcp_client::TcpClient;

pub struct ModbusClient {
    pub host: String,
    pub port: u16,
    pub tcp_client: TcpClient
}

#[allow(dead_code)]
impl ModbusClient {
    pub fn read_coils(&mut self, start_address: u16, count: u16) -> Vec<bool> {
        // Create request
        let mut request: Frame = Frame::default();
        *request.pdu() = Pdu::from_bytes([
            0x01,
            Utils::u16_to_u8(start_address)[0],
            Utils::u16_to_u8(start_address)[1],
            Utils::u16_to_u8(count)[0],
            Utils::u16_to_u8(count)[1],
        ].to_vec());
        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);

        // Send request
        let tmp = self.tcp_client.send(self.host.to_string(), self.port, request.to_bytes());

        // Read response
        if tmp.len() > 7 {
            let mut response: Frame = Frame::from_bytes(tmp);
            if !response.pdu().is_exception() {
                return Utils::bytes_to_bools(response.pdu().to_bytes()[2..].to_vec(), count as usize);
            }
        }
        [].to_vec()
    }

    pub fn read_discrete_inputs(&mut self, start_address: u16, count: u16) -> Vec<bool> {
        // Create request
        let mut request: Frame = Frame::default();
        *request.pdu() = Pdu::from_bytes([
            0x02,
            Utils::u16_to_u8(start_address)[0],
            Utils::u16_to_u8(start_address)[1],
            Utils::u16_to_u8(count)[0],
            Utils::u16_to_u8(count)[1],
        ].to_vec());
        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);

        // Send request
        let tmp = self.tcp_client.send(self.host.to_string(), self.port, request.to_bytes());

        // Read response
        if tmp.len() > 7 {
            let mut response: Frame = Frame::from_bytes(tmp);
            if !response.pdu().is_exception() {
                return Utils::bytes_to_bools(response.pdu().to_bytes()[2..].to_vec(), count as usize);
            }
        }
        [].to_vec()
    }

    pub fn read_holding_registers(&mut self, start_address: u16, count: u16) -> Vec<u16> {
        if count > 125 {
            panic!("Value for `count` can only go up to 0x7D (125)");
        }

        // Create request
        let mut request: Frame = Frame::default();
        *request.pdu() = Pdu::from_bytes([
            0x03,
            Utils::u16_to_u8(start_address)[0],
            Utils::u16_to_u8(start_address)[1],
            Utils::u16_to_u8(count)[0],
            Utils::u16_to_u8(count)[1],
        ].to_vec());
        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);

        // Send request
        let tmp = self.tcp_client.send(self.host.to_string(), self.port, request.to_bytes());

        // Read response
        if tmp.len() > 7 {
            let mut response: Frame = Frame::from_bytes(tmp);
            if !response.pdu().is_exception() {
                return Utils::u8_vec_to_u16_vec(response.pdu().to_bytes()[2..].to_vec());
            }
        }
        [].to_vec()
    }

    pub fn read_input_registers(&mut self, start_address: u16, count: u16) -> Vec<u16> {
        if count > 125 {
            panic!("Value for `count` can only go up to 0x7D (125)");
        }

        // Create request
        let mut request: Frame = Frame::default();
        *request.pdu() = Pdu::from_bytes([
            0x04,
            Utils::u16_to_u8(start_address)[0],
            Utils::u16_to_u8(start_address)[1],
            Utils::u16_to_u8(count)[0],
            Utils::u16_to_u8(count)[1],
        ].to_vec());
        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);

        // Send request
        let tmp = self.tcp_client.send(self.host.to_string(), self.port, request.to_bytes());

        // Read response
        if tmp.len() > 7 {
            let mut response: Frame = Frame::from_bytes(tmp);
            if !response.pdu().is_exception() {
                return Utils::u8_vec_to_u16_vec(response.pdu().to_bytes()[2..].to_vec());
            }
        }
        [].to_vec()
    }

    pub fn write_single_coil(&mut self, address: u16, status: bool) -> bool {
        // Create request
        let mut request: Frame = Frame::default();
        let coil_status: u16 = if status {
            0xFF00
        } else {
            0x0000
        };
        *request.pdu() = Pdu::from_bytes([
            0x05,
            Utils::u16_to_u8(address)[0],
            Utils::u16_to_u8(address)[1],
            Utils::u16_to_u8(coil_status)[0],
            Utils::u16_to_u8(coil_status)[1],
        ].to_vec());
        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);

        // Send request
        let tmp = self.tcp_client.send(self.host.to_string(), self.port, request.to_bytes());

        // Read response
        if tmp.len() > 7 {
            let mut response: Frame = Frame::from_bytes(tmp);
            response.pdu().get_function_code() == request.pdu().get_function_code()
        } else {
            false
        }
    }

    pub fn write_single_register(&mut self, address: u16, value: u16) -> bool {
        // Create request
        let mut request: Frame = Frame::default();

        *request.pdu() = Pdu::from_bytes([
            0x06,
            Utils::u16_to_u8(address)[0],
            Utils::u16_to_u8(address)[1],
            Utils::u16_to_u8(value)[0],
            Utils::u16_to_u8(value)[1],
        ].to_vec());
        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);

        // Send request
        let tmp = self.tcp_client.send(self.host.to_string(), self.port, request.to_bytes());

        // Read response
        if tmp.len() > 7 {
            let mut response: Frame = Frame::from_bytes(tmp);
            response.pdu().get_function_code() == request.pdu().get_function_code()
        } else {
            false
        }
    }

    pub fn write_multiple_coils(&mut self, start_address: u16, values: Vec<bool>) -> bool {
        // Create request
        let output_quantity = values.len() as u16;
        let input_bytes = Utils::bools_to_bytes(values);
        let number_of_bytes = input_bytes.len();
        let mut request: Frame = Frame::default();
        let mut request_data: Vec<u8> = [
            0x0F,
            Utils::u16_to_u8(start_address)[0],
            Utils::u16_to_u8(start_address)[1],
            Utils::u16_to_u8(output_quantity)[0],
            Utils::u16_to_u8(output_quantity)[1],
            number_of_bytes as u8,
        ].to_vec();
        request_data.extend(input_bytes);
        *request.pdu() = Pdu::from_bytes(request_data);
        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);

        // Send request
        let tmp = self.tcp_client.send(self.host.to_string(), self.port, request.to_bytes());

        // Read response
        if tmp.len() > 7 {
            let mut response: Frame = Frame::from_bytes(tmp);
            response.pdu().get_function_code() == request.pdu().get_function_code()
        } else {
            false
        }
    }

    pub fn write_multiple_registers(&mut self, start_address: u16, values: Vec<u16>) -> bool {
        // Create request
        let output_quantity = values.len() as u16;
        let input_bytes = Utils::u16_vec_to_u8_vec(values);
        let number_of_bytes = input_bytes.len();
        let mut request: Frame = Frame::default();
        let mut request_data: Vec<u8> = [
            0x10,
            Utils::u16_to_u8(start_address)[0],
            Utils::u16_to_u8(start_address)[1],
            Utils::u16_to_u8(output_quantity)[0],
            Utils::u16_to_u8(output_quantity)[1],
            number_of_bytes as u8,
        ].to_vec();
        request_data.extend(input_bytes);
        *request.pdu() = Pdu::from_bytes(request_data);
        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);

        // Send request
        let tmp = self.tcp_client.send(self.host.to_string(), self.port, request.to_bytes());

        // Read response
        if tmp.len() > 7 {
            let mut response: Frame = Frame::from_bytes(tmp);
            response.pdu().get_function_code() == request.pdu().get_function_code()
        } else {
            false
        }
    }

    pub fn send_raw(&mut self, data: Vec<u8>) -> Frame {
        let tmp = self.tcp_client.send(self.host.to_string(), self.port, data);
        if tmp.len() > 7 {
            Frame::from_bytes(tmp)
        } else {
            Frame::default()
        }
    }
}
