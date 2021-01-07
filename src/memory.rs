use crate::bus::Device;
use log::info;

pub enum Permission {
    Normal,
    ReadOnly,
    Invalid,
}

pub struct Memory {
    base: usize,
    memory: Vec<u8>,
    permission: Permission,
}

impl Memory {
    pub fn new(base: usize, size: usize, binary: Vec<u8>, perm: Permission) -> Box<Self> {
        let mut memory = binary.clone();
        let rest = size - memory.len();
        let mut empty = vec![0; rest];
        memory.append(&mut empty);
        Box::new(Self {
            base: base,
            memory: memory,
            permission: perm,
        })
    }

    pub fn new_empty(base: usize, size: usize, perm: Permission) -> Box<Self> {
        let memory = vec![0; size];
        Box::new(Self {
            base: base,
            memory: memory,
            permission: perm,
        })
    }

}

impl Device for Memory {
    fn load(&self, addr: u16) -> Result<u8, ()> {
        match self.permission {
            Permission::Normal | Permission::ReadOnly => {
                let addr = (addr as usize) - self.base;
                match self.memory.get(addr) {
                    Some(elem) => Ok(*elem),
                    None => Err(()),
                }
            },
            Permission::Invalid => {
                info!("Invalid load on address {:#X}", addr);
                Ok(0)
            },
        }
    }

    fn store(&mut self, addr: u16, value: u8) -> Result<(), ()> {
        match self.permission {
            Permission::Normal => {
                let addr = (addr as usize) - self.base;
                match self.memory.get_mut(addr) {
                    Some(elem) => {
                        *elem = value;
                        Ok(())
                    },
                    None => Err(()),
                }
            },
            Permission::ReadOnly => {
                Ok(())
            },
            Permission::Invalid => {
                info!("Invalid store to address {:#X}", addr);
                Ok(())
            },
        }
    }
    fn range(&self) -> (u16, u16) {
        let start = self.base as u16;
        let end = start + self.memory.len() as u16 - 1;
        (start, end)
    }
}
