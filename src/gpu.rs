use crate::bus::{Device, VRAM_START, VRAM_END};

pub struct Gpu {
    vram: Vec<u8>,
}

impl Gpu {
    pub fn new() -> Self {
        let ram = vec![0; (VRAM_END - VRAM_START + 1) as usize];
        Self {
            vram: ram,
        }
    }

    pub fn build_screen(&self, buffer: &mut Vec<u32>) {
    }
}

impl Device for Gpu {
    fn load(&self, addr: u16) -> Result<u8, ()> {
        match self.vram.get(addr as usize) {
            Some(elem) => Ok(*elem),
            None => Err(()),
        }
    }

    fn store(&mut self, addr: u16, value: u8) -> Result<(), ()> {
        match self.vram.get_mut(addr as usize) {
            Some(elem) => {
                *elem = value;
                Ok(())
            },
            None => Err(()),
        }
    }
}
