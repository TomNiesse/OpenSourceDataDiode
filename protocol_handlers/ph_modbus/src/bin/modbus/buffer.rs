use std::sync::{Mutex, Arc};

pub struct Buffer {
    data: Vec<Vec<u8>>
}

#[allow(dead_code)]
impl Buffer {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            data: vec![]
        }))
    }
    pub fn is_empty(&self) -> bool {
        self.data.len() <= 0
    }
    pub fn data(&self) -> &Vec<Vec<u8>> {
        &self.data
    }
    pub fn latest(&self) -> &Vec<u8> {
        &self.data[0]
    }
    pub fn add(&mut self, data: Vec<u8>) {
        self.data.push(data);
    }
    pub fn clean(&mut self) {
        self.data.remove(0);
    }
}
