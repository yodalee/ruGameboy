use crate::register::Register;
use crate::instruction::{Instruction, Target, Condition};
use crate::bus::Bus;

pub struct Cpu {
    regs: Register,
    sp: u16,
    pub pc: u16,
    bus: Bus,
}

impl Cpu {
    pub fn new(binary: Vec<u8>) -> Self {
        Self {
            regs: Register::default(),
            sp: 0,
            pc: 0x0100, // Starting point of execution
            bus: Bus::new(binary),
        }
    }

    pub fn fetch(&self) -> Result<u16, ()> {
        self.bus.load(self.pc, 8)
    }

    pub fn load(&self, addr: u16, size: u16) -> Result<u16, ()> {
        self.bus.load(addr, size)
    }

    pub fn store(&mut self, addr: u16, size: u16, value: u16) -> Result<(), ()> {
        self.bus.store(addr, size, value)
    }

    pub fn execute(&mut self, inst: Instruction) -> Result<u16, ()> {
        println!("{}", self.dump());
        let len = inst.len();
        match inst {
            Instruction::NOP => {},
            Instruction::JP => {
                let addr = self.load(self.pc + 1, 16)?;
                self.pc = addr;
            },
            Instruction::DI => {
                // disable interrupt, since we have no interrupt yet
            }
            Instruction::LDIMM16(target) => {
                let imm = self.load(self.pc + 1, 16)?;
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
                let addr = self.load(self.pc + 1, 16)?;
                self.store(addr, 8, self.regs.a as u16);
            }
            Instruction::LDA16 => {
                let addr = self.load(self.pc + 1, 16)?;
                self.regs.a = self.load(addr, 8)? as u8;
            }
            Instruction::LDIMM8(target) => {
                let imm = self.load(self.pc + 1, 8)? as u8;
                match target {
                    Target::A => self.regs.a = imm,
                    Target::B => self.regs.b = imm,
                    Target::C => self.regs.c = imm,
                    Target::D => self.regs.d = imm,
                    Target::E => self.regs.e = imm,
                    Target::H => self.regs.h = imm,
                    Target::L => self.regs.l = imm,
                    Target::HL => self.store(self.regs.get_hl(), 8, imm as u16)?,
                    _ => {
                        dbg!("Invalid target for instruction {:?}, target");
                        return Err(());
                    }
                }
            }
            Instruction::LD8A => {
                let addr = 0xff00 + (self.load(self.pc + 1, 8)?);
                self.store(addr, 8, self.regs.a as u16);
            }
            Instruction::LDA8 => {
                let addr = 0xff00 + (self.load(self.pc + 1, 8)?);
                self.regs.a = self.load(addr, 8)? as u8;
            }
            Instruction::LDRR(source, target) => {
                match (source, target) {
                    (Target::L, Target::A) => self.regs.a = self.regs.l,
                    (Target::H, Target::A) => self.regs.a = self.regs.h,
                    (Target::HLINC, Target::A) => {
                        self.store(self.regs.get_hl(), 8, self.regs.a as u16);
                        self.regs.set_hl(self.regs.get_hl() + 1);
                    },
                    (Target::HLDEC, Target::A) => {
                        self.store(self.regs.get_hl(), 8, self.regs.a as u16);
                        self.regs.set_hl(self.regs.get_hl() + 1);
                    },
                    (_, _) => {
                        dbg!("Invalid target for instruction {:?}, target");
                        return Err(());
                    }
                }
            }
            Instruction::CALL => {
                let addr = self.load(self.pc + 1, 16)?;
                self.store(self.sp-1, 16, self.pc + 2);
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
                    self.pc = self.load(self.sp + 1, 16)?;
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
                self.store(self.sp-1, 16, value);
                self.sp -= 2;
            }
            Instruction::POP(target) => {
                let value = self.load(self.sp+1, 16)?;
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
                    let offset = self.load(self.pc + 1, 8)? as i8;
                    self.pc = self.pc.wrapping_add(offset as u16);
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
