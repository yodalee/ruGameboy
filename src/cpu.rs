pub struct Cpu {
    regs: [u16; 4],
    sp: u16,
    pub pc: u16,
    memory: Vec<u8>,
}

pub type Inst = u16;

impl Cpu {
    pub fn new(binary: Vec<u8>) -> Self {
        Self {
            regs: [0u16; 4],
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
        for i in 0..4 {
            output.push_str(&format!("reg[{}] = {:#x}\t", i, self.regs[i]));
        }
        output.push_str(&format!("SP = {:#x}\t", self.sp));
        output.push_str(&format!("pc = {:#x}\n", self.pc));
        output
    }
}
