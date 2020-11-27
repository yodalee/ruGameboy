use crate::memory::Memory;
use crate::gpu::Gpu;

/// memory map of LR35902, xxx_START to xxx_END inclusive
pub const CATRIDGE_START: u16 = 0x0000;
pub const CATRIDGE_END:   u16 = 0x7fff;
pub const VRAM_START:     u16 = 0x8000;
pub const VRAM_END:       u16 = 0x9fff;
pub const RAM_START:      u16 = 0xc000;
pub const RAM_END:        u16 = 0xdfff;
pub const HRAM_START:     u16 = 0xff80;
pub const HRAM_END:       u16 = 0xfffe;

pub trait Device {
    fn load(&self, addr: u16) -> Result<u8, ()>;
    fn store(&mut self, addr: u16, value: u8) -> Result<(), ()>;
}

pub struct Bus {
    catridge: Memory,
    ram: Memory,
    pub gpu: Gpu,
    hram: Memory,
}

impl Bus {
    pub fn new(binary: Vec<u8>) -> Self {
        let catridge = Memory::new(binary);
        Self {
            catridge: catridge,
            ram: Memory::new_empty((RAM_END - RAM_START + 1) as usize),
            gpu: Gpu::new(),
            hram: Memory::new_empty((HRAM_END - HRAM_START + 1) as usize),
        }
    }

    fn load(&self, addr: u16) -> Result<u8, ()> {
        match addr {
            CATRIDGE_START ..= CATRIDGE_END => self.catridge.load(addr - CATRIDGE_START),
            VRAM_START ..= VRAM_END => self.gpu.load(addr - VRAM_START),
            RAM_START ..= RAM_END => self.ram.load(addr - RAM_START),
            HRAM_START ..= HRAM_END => self.hram.load(addr - HRAM_START),
            _ => Err(()),
        }
    }

    fn store(&mut self, addr: u16, value: u8) -> Result<(), ()> {
        match addr {
            CATRIDGE_START ..= CATRIDGE_END => self.catridge.store(addr - CATRIDGE_START, value),
            VRAM_START ..= VRAM_END => self.gpu.store(addr - VRAM_START, value),
            RAM_START ..= RAM_END => self.ram.store(addr - RAM_START, value),
            HRAM_START ..= HRAM_END => self.hram.store(addr - HRAM_START, value),
            _ => Err(()),
        }
    }

    pub fn load8(&self, addr: u16) -> Result<u8, ()> {
        self.load(addr)
    }

    pub fn load16(&self, addr: u16) -> Result<u16, ()> {
        let msb = self.load(addr+1)?;
        let lsb = self.load(addr)?;
        Ok(((msb as u16) << 8) | (lsb as u16))
    }

    pub fn store8(&mut self, addr: u16, value: u8) -> Result<(), ()> {
        self.store(addr, value)
    }

    pub fn store16(&mut self, addr: u16, value: u16) -> Result<(), ()> {
        self.store(addr, (value & 0xff) as u8)?;
        self.store(addr+1, ((value >> 8) & 0xff) as u8)?;
        Ok(())
    }
}
