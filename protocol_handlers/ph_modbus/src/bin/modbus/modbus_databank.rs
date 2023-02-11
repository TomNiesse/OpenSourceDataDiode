use std::sync::{Arc, Mutex};

pub struct ModbusDatabank {
    coils: Vec<bool>,
    inputs: Vec<bool>,
    holding_registers: Vec<u16>,
    input_registers: Vec<u16>
}

impl Default for ModbusDatabank {
    fn default() -> Self {
        const DATABANK_SIZE: usize = 65535;
        Self {
            coils: [false; DATABANK_SIZE].to_vec(),
            inputs: [false; DATABANK_SIZE].to_vec(),
            holding_registers: [0u16; DATABANK_SIZE].to_vec(),
            input_registers: [0u16; DATABANK_SIZE].to_vec()
        }
    }
}

#[allow(dead_code)]
impl ModbusDatabank {
    // Create a thread safe instance of ModbusDatabank::default()
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }
    pub fn read_coils(&self, start_address: u16, length: u16) -> Vec<bool> {
        if length > 0 && start_address as usize+length as usize <= self.coils.len() {
            return self.coils[start_address as usize..start_address as usize+length as usize].to_vec();
        }
        [].to_vec()
    }
    pub fn read_discrete_inputs(&self, start_address: u16, length: u16) -> Vec<bool> {
        if length > 0 && start_address as usize+length as usize <= self.inputs.len() {
            return self.inputs[start_address as usize..start_address as usize+length as usize].to_vec();
        }
        [].to_vec()
    }
    pub fn read_holding_registers(&self, start_address: u16, length: u16) -> Vec<u16> {
        if length > 0 && start_address as usize+length as usize <= self.holding_registers.len() {
            return self.holding_registers[start_address as usize..start_address as usize+length as usize].to_vec();
        }
        [].to_vec()
    }
    pub fn read_input_registers(&self, start_address: u16, length: u16) -> Vec<u16> {
        if length > 0 && start_address as usize+length as usize <= self.input_registers.len() {
            return self.input_registers[start_address as usize..start_address as usize+length as usize].to_vec();
        }
        [].to_vec()
    }

    pub fn write_coils(&mut self, start_address: u16, data: Vec<bool>) -> bool {
        if start_address as usize + data.len() <= self.coils.len() {
            for (index, value) in data.iter().enumerate() {
                self.coils[start_address as usize + index] = *value;
            }
            return true;
        }
        false
    }
    pub fn write_holding_registers(&mut self, start_address: u16, data: Vec<u16>) -> bool {
        if start_address as usize + data.len() <= self.holding_registers.len() {
            for (index, value) in data.iter().enumerate() {
                self.holding_registers[start_address as usize + index] = *value;
            }
            return true;
        }
        false
    }

    //
    // Unofficial helper functions are found below (these do not belong to the Modbus standard, but are needed in this project)
    //
    pub fn write_inputs(&mut self, start_address: u16, data: Vec<bool>) -> bool {
        if start_address as usize + data.len() <= self.inputs.len() {
            for (index, value) in data.iter().enumerate() {
                self.inputs[start_address as usize + index] = *value;
            }
            return true;
        }
        false
    }
    pub fn write_input_registers(&mut self, start_address: u16, data: Vec<u16>) -> bool {
        if start_address as usize + data.len() <= self.input_registers.len() {
            for (index, value) in data.iter().enumerate() {
                self.input_registers[start_address as usize + index] = *value;
            }
            return true;
        }
        false
    }
}
