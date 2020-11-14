use crate::memory::Memory;

const CATRIDGE_START: u16 = 0x0;
const CATRIDGE_END: u16 = 0x7fff;
const RAM_START: u16 = 0x8000;
const RAM_END: u16 = 0xffff;

pub struct Bus {
    catridge: Memory,
    ram: Memory,
}

impl Bus {
    pub fn new(binary: Vec<u8>) -> Self {
        let catridge = Memory::new(binary);
        let ram = Memory::new(vec![]);
        Self {
            catridge: catridge,
            ram: ram,
        }
    }

    pub fn load(&self, addr: u16, size: u16) -> Result<u16, ()> {
        match addr {
            CATRIDGE_START ..= CATRIDGE_END => self.catridge.load(addr - CATRIDGE_START, size),
            RAM_START ..= RAM_END => self.ram.load(addr - RAM_START, size),
            _ => Err(()),
        }
    }

    pub fn store(&mut self, addr: u16, size: u16, value: u16) -> Result<(), ()> {
        match addr {
            CATRIDGE_START ..= CATRIDGE_END => self.catridge.store(addr - CATRIDGE_START, size, value),
            RAM_START ..= RAM_END => self.ram.store(addr - RAM_START, size, value),
            _ => Err(()),
        }
    }
}
