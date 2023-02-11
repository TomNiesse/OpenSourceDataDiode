mod mbap;
pub use mbap::Mbap;
mod pdu;
pub use pdu::Pdu;

#[derive(Default)]
pub struct Frame {
    mbap: Mbap,
    pdu: Pdu
}

#[allow(dead_code)]
impl Frame {
    pub fn print(&self) {
        print!("MBAP: {:?}; PDU: {:?}; ", self.mbap.to_bytes(), self.pdu.to_bytes());
        match self.pdu.get_function_code() {
            0x01 => log::info!("(Read coils)"),
            0x02 => log::info!("(Read discrete inputs)"),
            0x03 => log::info!("(Read holding registers)"),
            0x04 => log::info!("(Read input registers)"),
            0x05 => log::info!("(Write single coil)"),
            0x06 => log::info!("(Write single register)"),
            0x0F => log::info!("(Write multiple coils)"),
            0x10 => log::info!("(Write multiple registers)"),
            0x16 => log::info!("(Mask write register)"),
            _    => log::info!("(Unknown function code)"),
        }

    }
    pub fn from_bytes(data: Vec<u8>) -> Self {
        if data.len() < 7 {
            return Self::default();
        }
        Self {
            mbap: Mbap::from_bytes(data[0..7].to_vec()),
            pdu: Pdu::from_bytes(data[7..].to_vec()),
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out: Vec<u8> = [].to_vec();
        out.extend(self.mbap.to_bytes());
        out.extend(self.pdu.to_bytes());
        out
    }
    pub fn is_valid(&self) -> bool {
        let mbap_valid = self.mbap.is_valid();
        let length_valid = self.mbap.get_length() as usize == self.pdu.to_bytes().len()+1;
        mbap_valid && length_valid
    }
    pub fn mbap(&mut self) -> &mut Mbap {
        &mut self.mbap
    }
    pub fn pdu(&mut self) -> &mut Pdu {
        &mut self.pdu
    }
}
