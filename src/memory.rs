use crate::bus::Device;

pub struct Memory {
    base: usize,
    memory: Vec<u8>,
}

impl Memory {
    pub fn new(base: usize, binary: Vec<u8>) -> Self {
        Self {
            base: base,
            memory: binary.clone()
        }
    }

    pub fn new_empty(base: usize, size: usize) -> Self {
        let memory = vec![0; size];
        Self {
            base: base,
            memory: memory,
        }
    }

}

impl Device for Memory {
    fn load(&self, addr: u16) -> Result<u8, ()> {
        let addr = (addr as usize) - self.base;
        match self.memory.get(addr) {
            Some(elem) => Ok(*elem),
            None => Err(()),
        }
    }

    fn store(&mut self, addr: u16, value: u8) -> Result<(), ()> {
        let addr = (addr as usize) - self.base;
        match self.memory.get_mut(addr) {
            Some(elem) => {
                *elem = value;
                Ok(())
            },
            None => Err(()),
        }
    }
}
