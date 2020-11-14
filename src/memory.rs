const MEMORY_SIZE: u16 = 0x8000;
const CATRIDGE_SIZE: u16 = 0x8000;

pub struct Memory {
    memory: Vec<u8>,
}

impl Memory {
    pub fn new(binary: Vec<u8>) -> Self {
        let mut memory = binary.clone();
        memory.resize(MEMORY_SIZE as usize, 0);
        Self { memory: memory }
    }

    pub fn load(&self, addr: u16, size: u16) -> Result<u16, ()> {
        match size {
            8 => Ok(self.load8(addr)),
            16 => Ok(self.load16(addr)),
            _ => Err(()),
        }
    }

    pub fn store(&mut self, addr: u16, size: u16, value: u16) -> Result<(), ()> {
        match size {
            8 => Ok(self.store8(addr, value)),
            16 => Ok(self.store16(addr, value)),
            _ => Err(()),
        }
    }

    pub fn load8(&self, addr: u16) -> u16 {
        self.memory[addr as usize] as u16
    }

    pub fn load16(&self, addr: u16) -> u16 {
        ((self.memory[(addr+1) as usize] as u16) << 8)
            | self.memory[addr as usize] as u16
    }

    pub fn store8(&mut self, addr: u16, value: u16) {
        self.memory[addr as usize] = value as u8
    }

    pub fn store16(&mut self, addr: u16, value: u16) {
        self.memory[addr as usize] = (value & 0xff) as u8;
        self.memory[(addr+1) as usize] = ((value >> 8) & 0xff) as u8;
    }
}
