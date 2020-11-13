use crate::register::Register;
use crate::instruction::{Instruction, Target, Condition};

pub struct Cpu {
    regs: Register,
    sp: u16,
    pub pc: u16,
    memory: Vec<u8>,
}

impl Cpu {
    pub fn new(binary: Vec<u8>) -> Self {
        Self {
            regs: Register::default(),
            sp: 0,
            pc: 0x0100, // Starting point of execution
            memory: binary,
        }
    }

    pub fn load8(&self) -> u8 {
        let index = self.pc as usize;
        let value = self.memory[index];
        value
    }

    pub fn loadimm8(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn loadimm16(&self, addr: u16) -> u16 {
        ((self.memory[(addr+1) as usize] as u16) << 8)
            | self.memory[addr as usize] as u16
    }

    pub fn store(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value
    }

    pub fn execute(&mut self, inst: Instruction) -> Result<u16, ()> {
        println!("{}", self.dump());
        let len = inst.len();
        match inst {
            Instruction::NOP => {},
            Instruction::JP => {
                let addr = self.loadimm16(self.pc + 1);
                self.pc = addr;
            },
            Instruction::DI => {
                // disable interrupt, since we have no interrupt yet
            }
            Instruction::LDIMM16(target) => {
                let imm = self.loadimm16(self.pc + 1);
                match target {
                    Target::BC => self.regs.set_bc(imm),
                    Target::DE => self.regs.set_de(imm),
                    Target::HL => self.regs.set_hl(imm),
                    Target::SP => self.sp = imm,
                    _ => {
                        dbg!("Invalid target for instruction {:?}, target");
                        return Err(());
                    }
                }
            }
            Instruction::LD16A => {
                let addr = self.loadimm16(self.pc + 1);
                self.store(addr, self.regs.a);
            }
            Instruction::LDA16 => {
                let addr = self.loadimm16(self.pc + 1);
                self.regs.a = self.loadimm8(addr);
            }
            Instruction::LDIMM8(target) => {
                let imm = self.loadimm8(self.pc + 1);
                match target {
                    Target::A => self.regs.a = imm,
                    Target::B => self.regs.b = imm,
                    Target::C => self.regs.c = imm,
                    Target::D => self.regs.d = imm,
                    Target::E => self.regs.e = imm,
                    Target::H => self.regs.h = imm,
                    Target::L => self.regs.l = imm,
                    Target::HL => self.store(self.regs.get_hl(), imm),
                    _ => {
                        dbg!("Invalid target for instruction {:?}, target");
                        return Err(());
                    }
                }
            }
            Instruction::LD8A => {
                let addr = 0xff00 + (self.loadimm8(self.pc + 1) as u16);
                self.store(addr, self.regs.a);
            }
            Instruction::LDA8 => {
                let addr = 0xff00 + (self.loadimm8(self.pc + 1) as u16);
                self.regs.a = self.loadimm8(addr);
            }
            Instruction::LDRR(source, target) => {
                match (source, target) {
                    (Target::L, Target::A) => self.regs.a = self.regs.l,
                    (Target::H, Target::A) => self.regs.a = self.regs.h,
                    (Target::HLINC, Target::A) => {
                        self.store(self.regs.get_hl(), self.regs.a);
                        self.regs.set_hl(self.regs.get_hl() + 1);
                    },
                    (Target::HLDEC, Target::A) => {
                        self.store(self.regs.get_hl(), self.regs.a);
                        self.regs.set_hl(self.regs.get_hl() + 1);
                    },
                    (_, _) => {
                        dbg!("Invalid target for instruction {:?}, target");
                        return Err(());
                    }
                }
            }
            Instruction::CALL => {
                let addr = self.loadimm16(self.pc + 1);
                self.store(self.sp, (((self.pc + 2) >> 8) & 0xff) as u8);
                self.store(self.sp-1, ((self.pc + 2) & 0xff) as u8);
                self.sp -= 2;
                self.pc = addr;
            }
            Instruction::RET(condition) => {
                let should_ret = match condition {
                    Condition::NotZero => !self.regs.f.zero,
                    Condition::Zero => self.regs.f.zero,
                    Condition::NotCarry => !self.regs.f.carry,
                    Condition::Carry => self.regs.f.carry,
                    Condition::Always => true,
                };
                if should_ret {
                    self.pc = self.loadimm16(self.sp + 1);
                    self.sp += 2;
                }
            }
            Instruction::PUSH(target) => {
                let value = match target {
                    Target::BC => self.regs.get_bc(),
                    Target::DE => self.regs.get_de(),
                    Target::HL => self.regs.get_hl(),
                    Target::AF => self.regs.get_af(),
                    _ => {
                        dbg!("Invalid target for instruction {:?}, target");
                        return Err(());
                    }
                };
                self.store(self.sp, ((value >> 8) & 0xff) as u8);
                self.store(self.sp-1, (value & 0xff) as u8);
                self.sp -= 2;
            }
            Instruction::POP(target) => {
                let value = self.loadimm16(self.sp+2);
                match target {
                    Target::BC => self.regs.set_bc(value),
                    Target::DE => self.regs.set_de(value),
                    Target::HL => self.regs.set_hl(value),
                    Target::AF => self.regs.set_af(value),
                    _ => {
                        dbg!("Invalid target for instruction {:?}, target");
                        return Err(());
                    }
                };
                self.sp += 2;
            }
            Instruction::JR(condition) => {
                let should_jump = match condition {
                    Condition::NotZero => !self.regs.f.zero,
                    Condition::Zero => self.regs.f.zero,
                    Condition::NotCarry => !self.regs.f.carry,
                    Condition::Carry => self.regs.f.carry,
                    Condition::Always => true,
                };
                if should_jump {
                    let offset = self.loadimm8(self.pc + 1) as i8;
                    self.pc = self.pc.wrapping_add(offset as u16);
                    return Ok(0);
                }
            }
            Instruction::INC(target) => {
                match target {
                    Target::BC => self.regs.set_bc(self.regs.get_bc() + 1),
                    Target::DE => self.regs.set_de(self.regs.get_de() + 1),
                    Target::HL => self.regs.set_hl(self.regs.get_hl() + 1),
                    Target::SP => self.sp += 1,
                    _ => {
                        dbg!("Invalid target for instruction {:?}, target");
                        return Err(());
                    }
                }
            }
        }
        Ok(len)
    }

    pub fn dump(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("register {:?}\n", self.regs));
        output.push_str(&format!("SP = {:#x}\n", self.sp));
        output.push_str(&format!("pc = {:#x}\n", self.pc));
        output
    }
}
