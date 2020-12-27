use crate::bus::Device;

pub const JOYPAD_ADDR: u16 = 0xff00;

pub enum JoypadKey {
    RIGHT,
    LEFT,
    UP,
    DOWN,
    A,
    B,
    SELECT,
    START,
}

pub struct Joypad {
    p14: u8,
    p15: u8,
    mask: u8
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            p14: 0x0F,
            p15: 0x0F,
            mask: 0x30,
        }
    }
}

impl Device for Joypad {
    fn load(&self, _addr: u16) -> Result<u8, ()> {
        match self.mask {
            0x20 => Ok(self.p14), // read P14: Left, Right, Up, Down
            0x10 => Ok(self.p15), // read P15: A, B, Select, Start
            _ => Ok(0x0F)     // other value just read nothing
        }
    }

    fn store(&mut self, _addr: u16, value: u8) -> Result<(), ()> {
        self.mask = value;
        Ok(())
    }
}
