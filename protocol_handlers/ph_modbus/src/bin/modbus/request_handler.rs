use std::sync::{Arc, Mutex};

#[path = "./frame.rs"] mod frame;
pub use frame::{Frame, Pdu, Mbap};
#[path = "./modbus_databank.rs"] mod modbus_databank;
pub use modbus_databank::ModbusDatabank;
#[path = "./utils.rs"] mod utils;
use utils::Utils;

pub struct RequestHandler {
    pub databank: Arc<Mutex<ModbusDatabank>>
}

#[allow(dead_code)]
impl RequestHandler {
    pub fn new(databank: Arc<Mutex<ModbusDatabank>>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self{
            databank
        }))
    }

    pub fn handle(&mut self, mut request: Frame) -> Frame {
        match request.pdu().get_function_code() as u8 {
            0x01 => self.read_coils(request),
            0x02 => self.read_discrete_inputs(request),
            0x03 => self.read_holding_registers(request),
            0x04 => self.read_input_registers(request),
            0x05 => self.write_single_coil(request),
            0x06 => self.write_single_register(request),
            0x0F => self.write_multiple_coils(request),
            0x10 => self.write_multiple_registers(request),
            0x16 => self.mask_write_register(request),
            0x17 => self.read_write_multiple_registers(request),
               _ => RequestHandler::function_not_implemented(request) // Respond with "function not implemented" exception
        }
    }

    // Create a panic response (function not implemented)
    pub fn function_not_implemented(mut request: Frame) -> Frame {
        let function_code = request.pdu().get_function_code();
        *request.pdu() = Pdu::from_bytes([function_code|0x80, 0x01].to_vec());
        request
    }
    // Create a panic response (server device failure)
    pub fn server_device_failure(mut request: Frame) -> Frame {
        let function_code = request.pdu().get_function_code();
        *request.pdu() = Pdu::from_bytes([function_code|0x80, 0x04].to_vec());
        request
    }

    pub fn read_coils(&self, mut request: Frame) -> Frame {
        // Parse frame
        let pdu_data = request.pdu().get_function_data();
        let start_address = Utils::u8_to_u16(pdu_data[0], pdu_data[1]);
        let count = Utils::u8_to_u16(pdu_data[2], pdu_data[3]);

        // Execute function
        let data = self.databank.lock().unwrap().read_coils(start_address, count);

        // Respond
        let mut response: Vec<u8> = [].to_vec();
        if !data.is_empty() {
            let mut out = Utils::bools_to_bytes(data);
            response.push(request.pdu().get_function_code());
            response.push(out.len() as u8);
            response.append(&mut out);
            *request.pdu() = Pdu::from_bytes(response);
        } else {
            response = [request.pdu().get_function_code()|0x80, 0x02].to_vec();
            *request.pdu() = Pdu::from_bytes(response);
        }
        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);

        request
    }

    pub fn read_discrete_inputs(&self, mut request: Frame) -> Frame {
        // Parse frame
        let pdu_data = request.pdu().get_function_data();
        let start_address = Utils::u8_to_u16(pdu_data[0], pdu_data[1]);
        let count = Utils::u8_to_u16(pdu_data[2], pdu_data[3]);

        // Execute function
        let data = self.databank.lock().unwrap().read_discrete_inputs(start_address, count);

        // Respond
        let mut response: Vec<u8> = [].to_vec();
        if !data.is_empty() {
            let mut out = Utils::bools_to_bytes(data);
            response.push(request.pdu().get_function_code());
            response.push(out.len() as u8);
            response.append(&mut out);
            *request.pdu() = Pdu::from_bytes(response);
        } else {
            response = [request.pdu().get_function_code()|0x80, 0x02].to_vec();
            *request.pdu() = Pdu::from_bytes(response);
        }
        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);

        request
    }

    pub fn read_holding_registers(&self, mut request: Frame) -> Frame {
        // Parse frame
        let pdu_data = request.pdu().get_function_data();
        let start_address = Utils::u8_to_u16(pdu_data[0], pdu_data[1]);
        let count = Utils::u8_to_u16(pdu_data[2], pdu_data[3]);

        // Execute function
        let data = self.databank.lock().unwrap().read_holding_registers(start_address, count);

        // Respond
        let mut response: Vec<u8> = [].to_vec();
        if !data.is_empty() {
            let mut out = Utils::u16_vec_to_u8_vec(data);
            response.push(request.pdu().get_function_code());
            response.push(out.len() as u8);
            response.append(&mut out);
            *request.pdu() = Pdu::from_bytes(response);
        } else {
            response = [request.pdu().get_function_code()|0x80, 0x02].to_vec();
            *request.pdu() = Pdu::from_bytes(response);
        }
        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);

        request
    }

    pub fn read_input_registers(&self, mut request: Frame) -> Frame {
        // Parse frame
        let pdu_data = request.pdu().get_function_data();
        let start_address = Utils::u8_to_u16(pdu_data[0], pdu_data[1]);
        let count = Utils::u8_to_u16(pdu_data[2], pdu_data[3]);

        // Execute function
        let data = self.databank.lock().unwrap().read_input_registers(start_address, count);

        // Respond
        let mut response: Vec<u8> = [].to_vec();
        if !data.is_empty() {
            let mut out = Utils::u16_vec_to_u8_vec(data);
            response.push(request.pdu().get_function_code());
            response.push(out.len() as u8);
            response.append(&mut out);
            *request.pdu() = Pdu::from_bytes(response);
        } else {
            response = [request.pdu().get_function_code()|0x80, 0x02].to_vec();
            *request.pdu() = Pdu::from_bytes(response);
        }
        let pdu_size = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_size+1);

        request
    }

    pub fn write_single_coil(&mut self, mut request: Frame) -> Frame {
        // Parse frame
        let pdu_data = request.pdu().get_function_data();
        let address = Utils::u8_to_u16(pdu_data[0], pdu_data[1]);
        let value = Utils::u8_to_u16(pdu_data[2], pdu_data[3]);

        // Execute function
        if value == 0x0000 || value == 0xFF00 {  // Only 0x0000 and 0xFF00 are valid values
            if !self.databank.lock().unwrap().write_coils(address, [value == 0xFF00].to_vec()) {
                // Respond with "illegal address" exception
                let response: Vec<u8> = [request.pdu().get_function_code()|0x80, 0x02].to_vec();
                *request.pdu() = Pdu::from_bytes(response);
                let pdu_size = request.pdu().to_bytes().len() as u16;
                request.mbap().set_length(pdu_size+1);
            }
        }
        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);

        // Return the request as-is, indicating the function executed fine
        request
    }

    pub fn write_single_register(&mut self, mut request: Frame) -> Frame {
        // Parse frame
        let pdu_data = request.pdu().get_function_data();
        let address = Utils::u8_to_u16(pdu_data[0], pdu_data[1]);
        let value = Utils::u8_to_u16(pdu_data[2], pdu_data[3]);

        // Execute function
        if !self.databank.lock().unwrap().write_holding_registers(address, [value].to_vec()) {
            // Respond with "illegal address" exception
            let function_code = request.pdu().get_function_code();
            *request.pdu() = Pdu::from_bytes([function_code|0x80, 0x02].to_vec());
        }
        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);

        // Return the request as-is, indicating the function executed fine
        request
    }

    pub fn write_multiple_coils(&mut self, mut request: Frame) -> Frame {
        // Parse frame
        let pdu_data = request.pdu().get_function_data();
        let start_address = Utils::u8_to_u16(pdu_data[0], pdu_data[1]);
        let count = Utils::u8_to_u16(pdu_data[2], pdu_data[3]);
        // let _byte_count = pdu_data[4];
        let in_data: Vec<bool> = Utils::bytes_to_bools(pdu_data[5..].to_vec(), count as usize);

        // Execute function
        let function_code = request.pdu().get_function_code();
        let mut response = request;
        if self.databank.lock().unwrap().write_coils(start_address, in_data) {
            // Respond with OK
            *response.pdu() = Pdu::from_bytes([
                function_code,
                Utils::u16_to_u8(start_address)[0], Utils::u16_to_u8(start_address)[1],
                Utils::u16_to_u8(count)[0], Utils::u16_to_u8(count)[1]
            ].to_vec());
        } else {
            // Respond with an exception
            *response.pdu() = Pdu::from_bytes([function_code|0x80, 0x02].to_vec());
        }
        let pdu_length = response.pdu().to_bytes().len() as u16;
        response.mbap().set_length(pdu_length+1);

        response
    }

    pub fn write_multiple_registers(&mut self, mut request: Frame) -> Frame {
        // Parse frame
        let pdu_data = request.pdu().get_function_data();
        let start_address = Utils::u8_to_u16(pdu_data[0], pdu_data[1]);
        let count = Utils::u8_to_u16(pdu_data[2], pdu_data[3]);
        // let byte_count = pdu_data[4];
        let in_data: Vec<u16> = Utils::u8_vec_to_u16_vec(pdu_data[5..].to_vec());

        // Execute function
        let function_code = request.pdu().get_function_code();
        let mut response = request;
        if self.databank.lock().unwrap().write_holding_registers(start_address, in_data) {
            // Respond with OK
            *response.pdu() = Pdu::from_bytes([
                function_code,
                Utils::u16_to_u8(start_address)[0], Utils::u16_to_u8(start_address)[1],
                Utils::u16_to_u8(count)[0], Utils::u16_to_u8(count)[1]
            ].to_vec());
        } else {
            // Respond with an exception
            *response.pdu() = Pdu::from_bytes([function_code|0x80, 0x02].to_vec());
        }
        let pdu_length = response.pdu().to_bytes().len() as u16;
        response.mbap().set_length(pdu_length+1);

        response
    }

    pub fn mask_write_register(&mut self, mut request: Frame) -> Frame {
        let pdu_data = request.pdu().get_function_data();
        let address = Utils::u8_to_u16(pdu_data[0], pdu_data[1]);
        let and_mask = Utils::u8_to_u16(pdu_data[2], pdu_data[3]);
        let or_mask = Utils::u8_to_u16(pdu_data[4], pdu_data[5]);

        let tmp: Vec<u16> = self.databank.lock().unwrap().read_holding_registers(address, 1);
        if !tmp.is_empty() {
            let mut current_value = tmp[0];
            current_value &= and_mask;
            current_value |= or_mask;
            self.databank.lock().unwrap().write_holding_registers(address, [current_value].to_vec());
        } else {
            *request.pdu() = Pdu::from_bytes([request.pdu().get_function_code()|0x80, 0x02].to_vec());
        }

        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);
        request
    }

    pub fn read_write_multiple_registers(&mut self, mut request: Frame) -> Frame {
        let pdu_data = request.pdu().get_function_data();
        let read_start_address = Utils::u8_to_u16(pdu_data[0], pdu_data[1]);
        let quantity_to_read = Utils::u8_to_u16(pdu_data[2], pdu_data[3]);
        let write_start_address = Utils::u8_to_u16(pdu_data[4], pdu_data[5]);
        // let quantity_to_write = Utils::u8_to_u16(pdu_data[6], pdu_data[7]);
        // let write_byte_count = pdu_data[8];
        let write_register_data: Vec<u16> = Utils::u8_vec_to_u16_vec(pdu_data[9..].to_vec());
        // Write
        self.databank.lock().unwrap().write_holding_registers(write_start_address, write_register_data);
        // Read
        let read_register_data = self.databank.lock().unwrap().read_holding_registers(read_start_address, quantity_to_read);
        // Respond
        if !read_register_data.is_empty() {
            let out = Utils::u16_vec_to_u8_vec(read_register_data);
            *request.pdu() = Pdu::from_bytes([request.pdu().get_function_code()].to_vec());
            request.pdu().extend([out.len() as u8].to_vec());
            request.pdu().extend(out);
        } else {
            *request.pdu() = Pdu::from_bytes([request.pdu().get_function_code()|0x80, 0x02].to_vec());
        }

        let pdu_length = request.pdu().to_bytes().len() as u16;
        request.mbap().set_length(pdu_length+1);
        request
    }
}
