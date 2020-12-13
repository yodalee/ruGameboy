use crate::memory::Memory;
use crate::gpu::{Gpu, LCDC};

use num_traits::FromPrimitive;
use num_derive::FromPrimitive;
use log::{error, info};

/// memory map of LR35902, xxx_START to xxx_END inclusive
pub const CATRIDGE_START: u16 = 0x0000;
pub const CATRIDGE_END:   u16 = 0x7fff;
pub const VRAM_START:     u16 = 0x8000;
pub const VRAM_END:       u16 = 0x9fff;
pub const RAM_START:      u16 = 0xc000;
pub const RAM_END:        u16 = 0xdfff;
pub const OAM_START:      u16 = 0xfe00;
pub const OAM_END:        u16 = 0xfe9f;
pub const UNUSABLE_START: u16 = 0xfea0;
pub const UNUSABLE_END:   u16 = 0xfeff;
pub const HRAM_START:     u16 = 0xff80;
pub const HRAM_END:       u16 = 0xfffe;

/// IO line, 0xff00 - 0xff7f
#[derive(FromPrimitive)]
enum IO {
    P1      = 0xff00,
    SB      = 0xff01,
    SC      = 0xff02,
    TMA     = 0xff06,
    TCA     = 0xff07,
    // consider move all NR line to one module
    NR10    = 0xff10,
    NR12    = 0xff12,
    NR14    = 0xff14,
    NR22    = 0xff17,
    NR24    = 0xff19,
    NR30    = 0xff1a,
    NR42    = 0xff21,
    NR44    = 0xff23,
    NR50    = 0xff24,
    NR51    = 0xff25,
    NR52    = 0xff26,
    LCDC    = 0xff40,
    STAT    = 0xff41,
    SCY     = 0xff42,
    SCX     = 0xff43,
    LY      = 0xff44,
    BGP     = 0xff47,
    OBP0    = 0xff48,
    OBP1    = 0xff49,
    WINY    = 0xff4a,
    WINX    = 0xff4b,
    Dummy7f = 0xff7f,
    Int     = 0xff0f,
    IntEnb  = 0xffff,
}

pub trait Device {
    fn load(&self, addr: u16) -> Result<u8, ()>;
    fn store(&mut self, addr: u16, value: u8) -> Result<(), ()>;
}

pub struct Bus {
    catridge: Memory,
    pub gpu: Gpu,
    ram: Memory,
    oam: Memory,
    hram: Memory,
    interruptflag: u8,
    interruptenb: u8,
}

impl Bus {
    pub fn new(binary: Vec<u8>) -> Self {
        let catridge = Memory::new(binary);
        Self {
            catridge: catridge,
            gpu: Gpu::new(),
            ram: Memory::new_empty((RAM_END - RAM_START + 1) as usize),
            oam: Memory::new_empty((OAM_END - OAM_START + 1) as usize),
            hram: Memory::new_empty((HRAM_END - HRAM_START + 1) as usize),
            interruptflag: 0,
            interruptenb: 0,
        }
    }

    fn load(&self, addr: u16) -> Result<u8, ()> {
        match addr {
            CATRIDGE_START ..= CATRIDGE_END => self.catridge.load(addr - CATRIDGE_START),
            VRAM_START ..= VRAM_END => self.gpu.load(addr - VRAM_START),
            RAM_START ..= RAM_END => self.ram.load(addr - RAM_START),
            OAM_START ..= OAM_END => self.oam.load(addr - OAM_START),
            UNUSABLE_START ..= UNUSABLE_END => {
                info!("Load at unusable address {:#x}", addr);
                Ok(0)
            }
            HRAM_START ..= HRAM_END => self.hram.load(addr - HRAM_START),
            _ => {
                // match IO line
                match FromPrimitive::from_u16(addr) {
                    Some(IO::LCDC) => Ok(self.gpu.lcdc.to_u8()),
                    Some(IO::SCY) => Ok(self.gpu.scy),
                    Some(IO::SCX) => Ok(self.gpu.scx),
                    Some(IO::LY) => Ok(self.gpu.line),
                    Some(IO::BGP) => Ok(self.gpu.bg_palette),
                    Some(IO::OBP0) => Ok(self.gpu.ob0_palette),
                    Some(IO::OBP1) => Ok(self.gpu.ob1_palette),
                    Some(IO::Int) => Ok(self.interruptflag),
                    Some(IO::IntEnb) => Ok(self.interruptenb),
                    Some(_) => Ok(0),
                    None => {
                        error!("Invalid load to address {:#x}", addr);
                        Err(())
                    }
                }
            }
        }
    }

    fn store(&mut self, addr: u16, value: u8) -> Result<(), ()> {
        match addr {
            CATRIDGE_START ..= CATRIDGE_END => self.catridge.store(addr - CATRIDGE_START, value),
            VRAM_START ..= VRAM_END => self.gpu.store(addr - VRAM_START, value),
            RAM_START ..= RAM_END => self.ram.store(addr - RAM_START, value),
            OAM_START ..= OAM_END => self.oam.store(addr - OAM_START, value),
            UNUSABLE_START ..= UNUSABLE_END => {
                info!("Write at unusable address {:#x}", addr);
                Ok(())
            }
            HRAM_START ..= HRAM_END => self.hram.store(addr - HRAM_START, value),
            _ => {
                // match IO line
                match FromPrimitive::from_u16(addr) {
                    Some(IO::LCDC) => self.gpu.lcdc = LCDC::from_u8(value),
                    Some(IO::SCY) => self.gpu.scy = value,
                    Some(IO::SCX) => self.gpu.scx = value,
                    Some(IO::LY) => self.gpu.line = 0,
                    Some(IO::BGP) => self.gpu.bg_palette = value,
                    Some(IO::OBP0) => self.gpu.ob0_palette = value,
                    Some(IO::OBP1) => self.gpu.ob1_palette = value,
                    Some(IO::Int) => self.interruptflag = value,
                    Some(IO::IntEnb) => self.interruptenb = value,
                    Some(_) => {},
                    None => {
                        error!("Invalid store to address {:#x}", addr);
                        return Err(())
                    }
                }
                Ok(())
            }
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
