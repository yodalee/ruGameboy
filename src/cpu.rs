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
                match &target {
                    &Target::BC => self.regs.set_bc(imm),
                    &Target::DE => self.regs.set_de(imm),
                    &Target::HL => self.regs.set_hl(imm),
                    &Target::SP => self.sp = imm,
                    _ => {
                        dbg!(format!("Invalid target for instruction {:?}", target));
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
                match (&source, &target) {
                    (&Target::B,  &Target::A) => self.regs.a = self.regs.b,
                    (&Target::L,  &Target::A) => self.regs.a = self.regs.l,
                    (&Target::H,  &Target::A) => self.regs.a = self.regs.h,
                    (&Target::B,  &Target::B) => self.regs.b = self.regs.b,
                    (&Target::C,  &Target::B) => self.regs.b = self.regs.c,
                    (&Target::D,  &Target::B) => self.regs.b = self.regs.d,
                    (&Target::E,  &Target::B) => self.regs.b = self.regs.e,
                    (&Target::H,  &Target::B) => self.regs.b = self.regs.h,
                    (&Target::L,  &Target::B) => self.regs.b = self.regs.l,
                    (&Target::HL, &Target::B) => self.regs.b = self.load(self.regs.get_hl(), 8)? as u8,
                    (&Target::A,  &Target::B) => self.regs.b = self.regs.a,
                    (&Target::B,  &Target::C) => self.regs.c = self.regs.b,
                    (&Target::C,  &Target::C) => self.regs.c = self.regs.c,
                    (&Target::D,  &Target::C) => self.regs.c = self.regs.d,
                    (&Target::E,  &Target::C) => self.regs.c = self.regs.e,
                    (&Target::H,  &Target::C) => self.regs.c = self.regs.h,
                    (&Target::L,  &Target::C) => self.regs.c = self.regs.l,
                    (&Target::HL, &Target::C) => self.regs.c = self.load(self.regs.get_hl(), 8)? as u8,
                    (&Target::A,  &Target::C) => self.regs.c = self.regs.a,
                    (&Target::B,  &Target::D) => self.regs.d = self.regs.b,
                    (&Target::C,  &Target::D) => self.regs.d = self.regs.c,
                    (&Target::D,  &Target::D) => self.regs.d = self.regs.d,
                    (&Target::E,  &Target::D) => self.regs.d = self.regs.e,
                    (&Target::H,  &Target::D) => self.regs.d = self.regs.h,
                    (&Target::L,  &Target::D) => self.regs.d = self.regs.l,
                    (&Target::HL, &Target::D) => self.regs.d = self.load(self.regs.get_hl(), 8)? as u8,
                    (&Target::A,  &Target::D) => self.regs.d = self.regs.a,
                    (&Target::B,  &Target::E) => self.regs.e = self.regs.b,
                    (&Target::C,  &Target::E) => self.regs.e = self.regs.c,
                    (&Target::D,  &Target::E) => self.regs.e = self.regs.d,
                    (&Target::E,  &Target::E) => self.regs.e = self.regs.e,
                    (&Target::H,  &Target::E) => self.regs.e = self.regs.h,
                    (&Target::L,  &Target::E) => self.regs.e = self.regs.l,
                    (&Target::HL, &Target::E) => self.regs.e = self.load(self.regs.get_hl(), 8)? as u8,
                    (&Target::A,  &Target::E) => self.regs.e = self.regs.a,
                    (&Target::B,  &Target::H) => self.regs.h = self.regs.b,
                    (&Target::C,  &Target::H) => self.regs.h = self.regs.c,
                    (&Target::D,  &Target::H) => self.regs.h = self.regs.d,
                    (&Target::E,  &Target::H) => self.regs.h = self.regs.e,
                    (&Target::H,  &Target::H) => self.regs.h = self.regs.h,
                    (&Target::L,  &Target::H) => self.regs.h = self.regs.l,
                    (&Target::HL, &Target::H) => self.regs.h = self.load(self.regs.get_hl(), 8)? as u8,
                    (&Target::A,  &Target::H) => self.regs.h = self.regs.a,
                    (&Target::B,  &Target::L) => self.regs.l = self.regs.b,
                    (&Target::C,  &Target::L) => self.regs.l = self.regs.c,
                    (&Target::D,  &Target::L) => self.regs.l = self.regs.d,
                    (&Target::E,  &Target::L) => self.regs.l = self.regs.e,
                    (&Target::H,  &Target::L) => self.regs.l = self.regs.h,
                    (&Target::L,  &Target::L) => self.regs.l = self.regs.l,
                    (&Target::HL, &Target::L) => self.regs.l = self.load(self.regs.get_hl(), 8)? as u8,
                    (&Target::A,  &Target::L) => self.regs.l = self.regs.a,
                    (&Target::HLINC, &Target::A) => {
                        self.store(self.regs.get_hl(), 8, self.regs.a as u16);
                        self.regs.set_hl(self.regs.get_hl() + 1);
                    },
                    (&Target::HLDEC, &Target::A) => {
                        self.store(self.regs.get_hl(), 8, self.regs.a as u16);
                        self.regs.set_hl(self.regs.get_hl() + 1);
                    },
                    (_, _) => {
                        dbg!(format!("Invalid target for instruction {:?} {:?}", source, target));
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
                        dbg!(format!("Invalid target for instruction {:?}", target));
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
                        dbg!(format!("Invalid target for instruction {:?}", target));
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
                        dbg!(format!("Invalid target for instruction {:?}", target));
                        return Err(());
                    }
                }
            }
            Instruction::ADD(target) => {
                let value = match target {
                    Target::B  => self.regs.b,
                    Target::C  => self.regs.c,
                    Target::D  => self.regs.d,
                    Target::E  => self.regs.e,
                    Target::H  => self.regs.h,
                    Target::L  => self.regs.l,
                    Target::HL => self.load(self.regs.get_hl(), 8)? as u8,
                    Target::A  => self.regs.a,
                    _ => {
                        dbg!(format!("Invalid target for instruction {:?}", target));
                        return Err(());
                    }
                };
                self.regs.f.subtract = false;
                self.regs.f.half_carry = (0x0f & self.regs.a) + (0x0f & value) > 0x0f;
                self.regs.f.carry = (self.regs.a as u16) + (value as u16) > 0xff;
                // note that we have to update regs.a and sum after check other flag
                self.regs.a = self.regs.a.wrapping_add(value);
                self.regs.f.zero = self.regs.a == 0;
            }
            Instruction::ADC(target) => {
                let value = match target {
                    Target::B  => self.regs.b,
                    Target::C  => self.regs.c,
                    Target::D  => self.regs.d,
                    Target::E  => self.regs.e,
                    Target::H  => self.regs.h,
                    Target::L  => self.regs.l,
                    Target::HL => self.load(self.regs.get_hl(), 8)? as u8,
                    Target::A  => self.regs.a,
                    _ => {
                        dbg!(format!("Invalid target for instruction {:?}", target));
                        return Err(());
                    }
                };
                let carry = if self.regs.f.carry { 1 } else { 0 };
                self.regs.f.subtract = false;
                self.regs.f.half_carry = (0x0f & self.regs.a) + (0x0f & value) + carry > 0x0f;
                self.regs.f.carry = (self.regs.a as u16) + (value as u16) + (carry as u16) > 0xff;
                // note that we have to update a after check flag
                self.regs.a = self.regs.a.wrapping_add(value).wrapping_add(carry);
                self.regs.f.zero = self.regs.a == 0;
            }
            Instruction::SUB(target) => {
                let value = match target {
                    Target::B  => self.regs.b,
                    Target::C  => self.regs.c,
                    Target::D  => self.regs.d,
                    Target::E  => self.regs.e,
                    Target::H  => self.regs.h,
                    Target::L  => self.regs.l,
                    Target::HL => self.load(self.regs.get_hl(), 8)? as u8,
                    Target::A  => self.regs.a,
                    _ => {
                        dbg!(format!("Invalid target for instruction {:?}", target));
                        return Err(());
                    }
                };
                self.regs.f.subtract = true;
                // FIXME: is half_carry and carry definition correct?
                self.regs.f.half_carry = (0x0f & self.regs.a) > (0x0f & value);
                self.regs.f.carry = self.regs.a > value;
                // note that we have to update regs.a and sum after check other flag
                self.regs.a = self.regs.a.wrapping_sub(value);
                self.regs.f.zero = self.regs.a == 0;
            }
            Instruction::SBC(target) => {
                let value = match target {
                    Target::B  => self.regs.b,
                    Target::C  => self.regs.c,
                    Target::D  => self.regs.d,
                    Target::E  => self.regs.e,
                    Target::H  => self.regs.h,
                    Target::L  => self.regs.l,
                    Target::HL => self.load(self.regs.get_hl(), 8)? as u8,
                    Target::A  => self.regs.a,
                    _ => {
                        dbg!(format!("Invalid target for instruction {:?}", target));
                        return Err(());
                    }
                };
                let carry = if self.regs.f.carry { 1 } else { 0 };
                self.regs.f.subtract = true;
                // FIXME: is half_carry and carry definition correct?
                self.regs.f.half_carry = (0x0f & self.regs.a) > (0x0f & value) + carry;
                self.regs.f.carry = (self.regs.a as u16) > (value as u16) + (carry as u16);
                // note that we have to update regs.a and sum after check other flag
                self.regs.a = self.regs.a.wrapping_sub(value).wrapping_sub(carry);
                self.regs.f.zero = self.regs.a == 0;
            }
            Instruction::AND(target) => {
                let value = match target {
                    Target::B  => self.regs.b,
                    Target::C  => self.regs.c,
                    Target::D  => self.regs.d,
                    Target::E  => self.regs.e,
                    Target::H  => self.regs.h,
                    Target::L  => self.regs.l,
                    Target::HL => self.load(self.regs.get_hl(), 8)? as u8,
                    Target::A  => self.regs.a,
                    _ => {
                        dbg!(format!("Invalid target for instruction {:?}", target));
                        return Err(());
                    }
                };
                self.regs.a &= value;
                self.regs.f.zero = self.regs.a == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = true;
                self.regs.f.carry = false;
            }
            Instruction::XOR(target) => {
                let value = match target {
                    Target::B  => self.regs.b,
                    Target::C  => self.regs.c,
                    Target::D  => self.regs.d,
                    Target::E  => self.regs.e,
                    Target::H  => self.regs.h,
                    Target::L  => self.regs.l,
                    Target::HL => self.load(self.regs.get_hl(), 8)? as u8,
                    Target::A  => self.regs.a,
                    _ => {
                        dbg!(format!("Invalid target for instruction {:?}", target));
                        return Err(());
                    }
                };
                self.regs.a ^= value;
                self.regs.f.zero = self.regs.a == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                self.regs.f.carry = false;
            }
            Instruction::OR(target) => {
                let value = match target {
                    Target::B  => self.regs.b,
                    Target::C  => self.regs.c,
                    Target::D  => self.regs.d,
                    Target::E  => self.regs.e,
                    Target::H  => self.regs.h,
                    Target::L  => self.regs.l,
                    Target::HL => self.load(self.regs.get_hl(), 8)? as u8,
                    Target::A  => self.regs.a,
                    _ => {
                        dbg!(format!("Invalid target for instruction {:?}", target));
                        return Err(());
                    }
                };
                self.regs.a |= value;
                self.regs.f.zero = self.regs.a == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                self.regs.f.carry = false;
            }
            Instruction::CMP(target) => {
                let value = match target {
                    Target::B  => self.regs.b,
                    Target::C  => self.regs.c,
                    Target::D  => self.regs.d,
                    Target::E  => self.regs.e,
                    Target::H  => self.regs.h,
                    Target::L  => self.regs.l,
                    Target::HL => self.load(self.regs.get_hl(), 8)? as u8,
                    Target::A  => self.regs.a,
                    _ => {
                        dbg!(format!("Invalid target for instruction {:?}", target));
                        return Err(());
                    }
                };
                self.regs.f.zero = self.regs.a == value;
                self.regs.f.subtract = true;
                self.regs.f.half_carry = (0x0f & self.regs.a) > (0x0f & value);
                self.regs.f.carry = self.regs.a < value;
            }
        }
        Ok(len)
    }

    pub fn dump(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Register {{ {} }}\n", self.regs));
        output.push_str(&format!("SP = {:#x}\t", self.sp));
        output.push_str(&format!("pc = {:#x}\t", self.pc));
        let byte = self.load(self.pc, 8).unwrap() as u8;
        output.push_str(&format!("byte = {:#x}\t", byte));
        output.push_str(&format!("inst = {:?}", Instruction::from_byte(byte)));
        output
    }
}
