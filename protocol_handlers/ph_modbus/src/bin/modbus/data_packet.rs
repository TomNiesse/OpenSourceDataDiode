#[path = "./utils.rs"]
mod utils;
pub use utils::Utils;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DataType {
    ModbusCommand = 0,
    CoilValue = 1,
    InputValue = 2,
    HoldingRegisterValue = 3,
    InputRegisterValue = 4,
    Undefined = -1
}

pub struct DataPacket {
    pub datatype: DataType,
    pub id: u16,
    pub value: Vec<u8>
}

impl Default for DataPacket {
    fn default() -> Self {
        Self{
            datatype: DataType::Undefined,
            id: 0,
            value: vec![]
        }
    }
}

#[allow(dead_code)]
impl DataPacket {
    pub fn new(datatype: DataType, data: Vec<u8>, id: u16) -> Self {
        Self {
            value: data,
            datatype,
            id
        }
    }
    pub fn get_id(&self) -> u16 {
        self.id
    }
    pub fn get_value(&self) -> &Vec<u8> {
        &self.value
    }
    pub fn from_bytes(data: Vec<u8>) -> Self {
        if data.len() < 6 {
            return Self::default();
        }

        let datatype = match data[0] {
            0 => DataType::ModbusCommand,
            1 => DataType::CoilValue,
            2 => DataType::InputValue,
            3 => DataType::HoldingRegisterValue,
            4 => DataType::InputRegisterValue,
            _ => DataType::Undefined
        };
        let id = Utils::u8_to_u16(data[1], data[2]);
        let value = data[3..].to_vec();

        Self {
            datatype,
            id,
            value
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out: Vec<u8> = vec![];
        out.push(self.datatype as u8);
        out.push(Utils::u16_to_u8(self.id)[0]);
        out.push(Utils::u16_to_u8(self.id)[1]);
        out.extend(&self.value);
        out
    }
}
