#[path = "./utils.rs"] mod utils;
use utils::Utils;

pub struct Mbap {
    data: Vec<u8>
}

impl Default for Mbap {
    fn default() -> Self {
        Self {
            data: [0u8;7].to_vec()
        }
    }
}

#[allow(dead_code)]
impl Mbap {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data: data[0..7].to_vec()
        }
    }
    pub fn to_bytes(&self) -> &Vec<u8> {
        &self.data
    }
    pub fn is_valid(&self) -> bool {
        if self.data.len() != 7 {
            log::info!("MBAP LENGTH IS INVALID");
            return false;
        }

        if self.get_protocol_id() != 0x0000 {
            log::info!("PROTOCOL IS INVALID");
            return false;
        }

        true
    }

    pub fn get_transaction_id(&self) -> u16 {
        Utils::u8_to_u16(self.data[0], self.data[1])
    }
    pub fn get_protocol_id(&self) -> u16 {
        Utils::u8_to_u16(self.data[2], self.data[3])
    }
    pub fn get_length(&self) -> u16 {
        Utils::u8_to_u16(self.data[4], self.data[5])
    }
    pub fn get_unit_id(&self) -> u8 {
        self.data[6]
    }

    pub fn set_transaction_id(&mut self, value: u16) {
        self.data[0] = Utils::u16_to_u8(value)[0];
        self.data[1] = Utils::u16_to_u8(value)[1];
    }
    pub fn set_protocol_id(&mut self, value: u16) {
        self.data[2] = Utils::u16_to_u8(value)[0];
        self.data[3] = Utils::u16_to_u8(value)[1];
    }
    pub fn set_length(&mut self, value: u16) {
        self.data[4] = Utils::u16_to_u8(value)[0];
        self.data[5] = Utils::u16_to_u8(value)[1];
    }
    pub fn set_unit_id(&mut self, value: u8) {
        self.data[6] = value;
    }
}
