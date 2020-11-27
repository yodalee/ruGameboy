use crate::bus::Device;

pub struct Memory {
    memory: Vec<u8>,
}

impl Memory {
    pub fn new(binary: Vec<u8>) -> Self {
        Self {
            memory: binary.clone()
        }
    }

    pub fn new_empty(size: usize) -> Self {
        let memory = vec![0; size];
        Self {
            memory: memory,
        }
    }

}

impl Device for Memory {
    fn load(&self, addr: u16) -> Result<u8, ()> {
        match self.memory.get(addr as usize) {
            Some(elem) => Ok(*elem),
            None => Err(()),
        }
    }

    fn store(&mut self, addr: u16, value: u8) -> Result<(), ()> {
        match self.memory.get_mut(addr as usize) {
            Some(elem) => {
                *elem = value;
                Ok(())
            },
            None => Err(()),
        }
    }
}
