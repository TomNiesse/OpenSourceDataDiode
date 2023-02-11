pub struct Utils {}

#[allow(dead_code)]
impl Utils {
    pub fn u8_to_u16(high: u8, low: u8) -> u16 {
        ((high as u16) << 8) | low as u16
    }
    pub fn u16_to_u8(value: u16) -> [u8; 2] {
        [(value >> 8) as u8, value as u8]
    }
    pub fn u8_vec_to_u16_vec(values: Vec<u8>) -> Vec<u16> {
        let mut in_data = values;
        let mut out: Vec<u16> = Vec::with_capacity(in_data.len()/2);
        while !in_data.is_empty() {
            if in_data.len() >= 2 {
                out.push(Utils::u8_to_u16(in_data[0], in_data[1]));
                in_data.remove(0);
                in_data.remove(0);
            } else {
                out.push(in_data[0] as u16);
                break;
            }
        }

        out
    }
    pub fn u16_vec_to_u8_vec(values: Vec<u16>) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(values.len()*2);
        for item in values {
            out.push(Utils::u16_to_u8(item)[0]);
            out.push(Utils::u16_to_u8(item)[1]);
        }
        out
    }
    pub fn bools_to_bytes(values: Vec<bool>) -> Vec<u8> {
        let mut in_data: Vec<bool> = values;
        let mut out: Vec<u8> = Vec::with_capacity(in_data.len()/8);

        while !in_data.is_empty() {
            let mut byte: u8 = 0x00;
            for bit in 0..8 {
                byte |= (in_data[0] as u8) << bit;
                in_data.remove(0);
                if in_data.is_empty() {
                    break;
                }
            }
            out.push(byte);
        }

        out
    }
    pub fn bytes_to_bools(values: Vec<u8>, number_of_bools: usize) -> Vec<bool> {
        let mut out: Vec<bool> = Vec::with_capacity(number_of_bools);

        for bit in 0..number_of_bools {
            let bit: u8 = ((1 << (bit % 8)) & (values[bit/8])) >> (bit % 8);
            out.push(bit != 0);
        }

        out
    }
}
