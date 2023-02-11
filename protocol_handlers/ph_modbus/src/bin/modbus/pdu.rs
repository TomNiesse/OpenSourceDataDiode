pub struct Pdu {
    data: Vec<u8>
}

impl Default for Pdu {
    fn default() -> Self {
        Self {
            data: [0u8;2].to_vec()
        }
    }
}

#[allow(dead_code)]
impl Pdu {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data
        }
    }
    pub fn to_bytes(&self) -> &Vec<u8> {
        &self.data
    }
    pub fn extend(&mut self, data: Vec<u8>) {
        self.data.extend(data);
    }
    pub fn get_function_code(&self) -> u8 {
        self.data[0]
    }
    pub fn get_function_data(&self) -> Vec<u8> {
        self.data[1..].to_vec()
    }
    pub fn get_exception(&self) -> u8 {
        self.data[1]
    }
    pub fn is_exception(&self) -> bool {
        (self.get_function_code() & 0x80) == 0x80
    }
    pub fn build_exception(&mut self, function_code: u8, exception: u8) {
        self.resize(2);
        self.data[0] = function_code | 0x80;
        self.data[1] = exception;
    }
    fn resize(&mut self, size: u8) {
        self.data.resize(size as usize,0);
    }
}
