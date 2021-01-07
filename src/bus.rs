use crate::memory::{Memory, Permission};
use crate::gpu::{Gpu, LCDC, VRAM_START, VRAM_END, OAM_START, OAM_END};
use crate::timer::{Timer, TIMER_START, TIMER_END};
use crate::joypad::{Joypad, JOYPAD_ADDR};

use num_traits::FromPrimitive;
use num_derive::FromPrimitive;
use log::{error, info, debug};

/// memory map of LR35902, xxx_START to xxx_END inclusive
const CATRIDGE_START: u16 = 0x0000;
const CATRIDGE_END:   u16 = 0x7fff;
const RAM_START:      u16 = 0xc000;
const RAM_END:        u16 = 0xdfff;
const UNUSABLE_START: u16 = 0xfea0;
const UNUSABLE_END:   u16 = 0xfeff;
const HRAM_START:     u16 = 0xff80;
const HRAM_END:       u16 = 0xfffe;
const INT:            u16 = 0xff0f;
const INTENB:         u16 = 0xffff;

/// Bit offset of interrupt register
const VBLANK_SHIFT: u8 = 0;
const LCDC_SHIFT: u8 = 1;
const TIMER_SHIFT: u8 = 2;
const SERIAL_SHIFT: u8 = 3;
const JOYPAD_SHIFT: u8 = 4;

#[derive(Debug,Default)]
pub struct InterruptFlag {
    // vblank on/off
    pub vblank: bool,
    // LCDC on/off
    pub lcdc: bool,
    // timer on/off
    pub timer: bool,
    // serial on/off
    pub serial: bool,
    // serial on/off
    pub joypad: bool,
}

impl std::convert::From<&InterruptFlag> for u8 {
    fn from(flag: &InterruptFlag) -> Self {
        ( if flag.vblank { 1 << VBLANK_SHIFT } else { 0 } ) |
        ( if flag.lcdc   { 1 << LCDC_SHIFT   } else { 0 } ) |
        ( if flag.timer  { 1 << TIMER_SHIFT  } else { 0 } ) |
        ( if flag.serial { 1 << SERIAL_SHIFT } else { 0 } ) |
        ( if flag.joypad { 1 << JOYPAD_SHIFT } else { 0 } )
    }
}

impl std::convert::From<u8> for InterruptFlag {
    fn from(byte: u8) -> Self {
        InterruptFlag {
            vblank: ((byte >> VBLANK_SHIFT) & 0b1) != 0,
            lcdc:   ((byte >> LCDC_SHIFT  ) & 0b1) != 0,
            timer:  ((byte >> TIMER_SHIFT ) & 0b1) != 0,
            serial: ((byte >> SERIAL_SHIFT) & 0b1) != 0,
            joypad: ((byte >> JOYPAD_SHIFT) & 0b1) != 0,
        }
    }
}

/// IO line, 0xff00 - 0xff7f
#[derive(FromPrimitive)]
enum IO {
    SB      = 0xff01,
    SC      = 0xff02,
    //TODO move all NR line from 0xff10 to 0xff3f one module
    NR10    = 0xff10,
    NR11    = 0xff11,
    NR12    = 0xff12,
    NR13    = 0xff13,
    NR14    = 0xff14,
    NR21    = 0xff16,
    NR22    = 0xff17,
    NR23    = 0xff18,
    NR24    = 0xff19,
    NR30    = 0xff1a,
    NR31    = 0xff1b,
    NR32    = 0xff1c,
    NR33    = 0xff1d,
    NR34    = 0xff1e,
    NR41    = 0xff20,
    NR42    = 0xff21,
    NR43    = 0xff22,
    NR44    = 0xff23,
    NR50    = 0xff24,
    NR51    = 0xff25,
    NR52    = 0xff26,
    WAV0    = 0xff30,
    WAV1    = 0xff31,
    WAV2    = 0xff32,
    WAV3    = 0xff33,
    WAV4    = 0xff34,
    WAV5    = 0xff35,
    WAV6    = 0xff36,
    WAV7    = 0xff37,
    WAV8    = 0xff38,
    WAV9    = 0xff39,
    WAVa    = 0xff3a,
    WAVb    = 0xff3b,
    WAVc    = 0xff3c,
    WAVd    = 0xff3d,
    WAVe    = 0xff3e,
    WAVf    = 0xff3f,
    LCDC    = 0xff40,
    STAT    = 0xff41,
    SCY     = 0xff42,
    SCX     = 0xff43,
    LY      = 0xff44,
    DMA     = 0xff46,
    BGP     = 0xff47,
    OBP0    = 0xff48,
    OBP1    = 0xff49,
    WINY    = 0xff4a,
    WINX    = 0xff4b,
    Dummy7f = 0xff7f,
}

pub trait Device {
    fn load(&self, addr: u16) -> Result<u8, ()>;
    fn store(&mut self, addr: u16, value: u8) -> Result<(), ()>;
    fn range(&self) -> (u16, u16);
}

pub struct Bus {
    storage: Vec<Box<dyn Device>>,
    pub gpu: Gpu,
    pub timer: Timer,
    pub joypad: Joypad,
    pub interruptenb: InterruptFlag,
}

impl Bus {
    pub fn new(binary: Vec<u8>) -> Self {
        let catridge = Memory::new(CATRIDGE_START as usize, (CATRIDGE_END - CATRIDGE_START + 1) as usize, binary, Permission::ReadOnly);
        let ram = Memory::new_empty(RAM_START as usize, (RAM_END - RAM_START + 1) as usize, Permission::Normal);
        let hram = Memory::new_empty(HRAM_START as usize, (HRAM_END - HRAM_START + 1) as usize, Permission::Normal);
        let unusable = Memory::new_empty(UNUSABLE_START as usize, (UNUSABLE_END - UNUSABLE_START + 1) as usize, Permission::Invalid);
        let mut storage:Vec<Box<dyn Device>> = Vec::new();
        storage.push(catridge);
        storage.push(ram);
        storage.push(hram);
        storage.push(unusable);
        Self {
            storage: storage,
            gpu: Gpu::new(),
            timer: Timer::new(),
            joypad: Joypad::new(),
            interruptenb: Default::default(),
        }
    }

    fn load_interrupt(&self) -> u8 {
       ( if self.gpu.is_interrupt   { 1 << VBLANK_SHIFT } else { 0 } ) |
       ( if self.timer.is_interrupt { 1 << TIMER_SHIFT  } else { 0 } )
    }

    fn store_interrupt(&mut self, value: u8) {
        self.gpu.is_interrupt   = (value >> VBLANK_SHIFT) & 0x1 != 0;
        self.timer.is_interrupt = (value >> TIMER_SHIFT)  & 0x4 != 0;
    }

    fn find_storage(&self, addr: u16) -> Option<&Box<dyn Device>> {
        let matched: Vec<&Box<dyn Device>> = self.storage
                                                .iter()
                                                .filter(|dev| {
                                                    let (start, end) = dev.range();
                                                    start <= addr && addr <= end
                                                }).collect();
        match matched.len() {
            0 => return None,
            1 => return Some(matched[0]),
            _ => {
                error!("Multiple device defined on address {:#X}", addr);
                std::process::exit(1);
            },
        }
    }

    fn find_device(&self, addr: u16) -> Option<&dyn Device> {
        match addr {
            VRAM_START ..= VRAM_END => Some(&self.gpu),
            OAM_START ..= OAM_END => Some(&self.gpu),
            TIMER_START ..= TIMER_END => Some(&self.timer),
            JOYPAD_ADDR => Some(&self.joypad),
            _ => return None,
        }
    }

    fn load(&self, addr: u16) -> Result<u8, ()> {
        if let Some(dev) = self.find_storage(addr) {
            return dev.load(addr);
        }

        match self.find_device(addr) {
            Some(dev) => dev.load(addr),
            None => match addr {
                INT => Ok(self.load_interrupt()),
                INTENB => Ok(u8::from(&self.interruptenb)),
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
                        Some(_) => {
                            info!("Unimplemented load on address {:#X}", addr);
                            Ok(0)
                        },
                        None => {
                            error!("Invalid load on address {:#X}", addr);
                            Err(())
                        }
                    }
                }
            }
        }
    }

    fn find_storage_mut(&mut self, addr: u16) -> Option<&mut Box<dyn Device>> {
        let mut matched: Vec<&mut Box<dyn Device>> = self.storage
                                                .iter_mut()
                                                .filter(|dev| {
                                                    let (start, end) = dev.range();
                                                    start <= addr && addr <= end
                                                }).collect();
        match matched.len() {
            0 => None,
            1 => {
                Some(matched.pop().unwrap())
            },
            _ => {
                error!("Multiple device defined on address {:#X}", addr);
                std::process::exit(1);
            },
        }
    }

    fn find_device_mut(&mut self, addr: u16) -> Option<&mut dyn Device> {
        match addr {
            VRAM_START ..= VRAM_END => Some(&mut self.gpu),
            OAM_START ..= OAM_END => Some(&mut self.gpu),
            TIMER_START ..= TIMER_END => Some(&mut self.timer),
            JOYPAD_ADDR => Some(&mut self.joypad),
            _ => return None,
        }
    }

    fn store(&mut self, addr: u16, value: u8) -> Result<(), ()> {
        if let Some(dev) = self.find_storage_mut(addr) {
            return dev.store(addr, value);
        }

        match self.find_device_mut(addr) {
            Some(dev) => dev.store(addr, value),
            None => match addr {
                INT => Ok(self.store_interrupt(value)),
                INTENB => Ok(self.interruptenb = InterruptFlag::from(value)),
                _ => {
                    // match IO line
                    match FromPrimitive::from_u16(addr) {
                        Some(IO::LCDC) => self.gpu.lcdc = LCDC::from_u8(value),
                        Some(IO::SCY) => self.gpu.scy = value,
                        Some(IO::SCX) => self.gpu.scx = value,
                        Some(IO::LY) => self.gpu.line = 0,
                        Some(IO::DMA) => self.dma(value),
                        Some(IO::BGP) => self.gpu.bg_palette = value,
                        Some(IO::OBP0) => self.gpu.ob0_palette = value,
                        Some(IO::OBP1) => self.gpu.ob1_palette = value,
                        Some(_) => {},
                        None => {
                            error!("Invalid store to address {:#X}", addr);
                            return Err(())
                        }
                    }
                    Ok(())
                }
            }
        }
    }

    fn dma(&mut self, value: u8) {
        /* dma copy 40 * 28 bits data to OAM zone 0xFE00-0xFE9F
         * each sprite takes 28 bits space (note that 4 bits are not used in each sprite)
         * the source address can be designated every 0x100 from 0x0000 to 0xF100.
         * Depend on the value stored to dma IO line:
         * 0x00 -> 0x0000
         * 0x01 -> 0x0100
         * ...
         */
        let addr = (value as u16) << 8;
        // copy memory to OAM
        for i in 0..(40 * 4) {
            let byte = self.load(addr + i).unwrap();
            self.store(OAM_START + i, byte).unwrap();
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
