use crate::register::Register;
use crate::opcode;

pub struct Cpu {
    regs: Register,
    sp: u16,
    pub pc: u16,
    memory: Vec<u8>,
}

pub type Inst = u8;

impl Cpu {
    pub fn new(binary: Vec<u8>) -> Self {
        Self {
            regs: Register::default(),
            sp: 0,
            pc: 0x0100, // Starting point of execution
            memory: binary,
        }
    }

    pub fn fetch(&self) -> Inst {
        let index = self.pc as usize;
        self.memory[index]
    }

    pub fn load8(&self) -> u16 {
        let index = self.pc as usize;
        self.memory[index+1] as u16
    }

    pub fn load16(&self) -> u16 {
        let index = self.pc as usize;
        ((self.memory[index+2] as u16) << 8) | self.memory[index+1] as u16
    }

    pub fn execute(&mut self, inst: Inst) -> Result<u16, ()> {
        match inst {
            opcode::NOP => {
                Ok(1)
            },
            opcode::JP => {
                let addr = self.load16();
                self.pc = addr;
                Ok(0)
            },
            opcode::DI => {
                // disable interrupt, since we have no interrupt yet
                Ok(1)
            }
            _ => {
                dbg!(&format!("Unsupport instruction {:#x}", inst));
                return Err(())
            }
        }
    }

    pub fn dump(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("register {:?}\n", self.regs));
        output.push_str(&format!("SP = {:#x}\n", self.sp));
        output.push_str(&format!("pc = {:#x}\n", self.pc));
        output
    }
}
