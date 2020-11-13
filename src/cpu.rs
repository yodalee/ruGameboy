use crate::register::Register;

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
        0
    }

    pub fn execute(&mut self, inst: Inst) {
    }

    pub fn dump(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("register {:?}\t", self.regs));
        output.push_str(&format!("SP = {:#x}\t", self.sp));
        output.push_str(&format!("pc = {:#x}\n", self.pc));
        output
    }
}
