use log::{debug, info};

use crate::register::Register;
use crate::instruction::{Instruction, Target, Condition, CBInstruction};
use crate::bus::Bus;

enum DataSize {
    Byte,
    Word,
}

#[derive(Eq,PartialEq,Clone,Copy)]
pub enum InterruptState {
    IDisable,
    IEnable,
    IDisableNext,
    IEnableNext,
}

impl Default for InterruptState {
    fn default() -> Self { InterruptState::IDisable }
}

pub struct Cpu {
    regs: Register,
    sp: u16,
    pub pc: u16,
    pub bus: Bus,
    interrupt_state: InterruptState,
}

impl Cpu {
    pub fn new(binary: Vec<u8>) -> Self {
        Self {
            regs: Register::default(),
            sp: 0,
            pc: 0x0100, // Starting point of execution
            bus: Bus::new(binary),
            interrupt_state: InterruptState::default(),
        }
    }

    pub fn fetch(&mut self) -> Result<u16, ()> {
        let byte = self.load(self.pc, DataSize::Word);
        self.pc += 1;
        byte
    }

    fn load(&self, addr: u16, size: DataSize) -> Result<u16, ()> {
        match size {
            DataSize::Byte => self.bus.load8(addr).map(|v| v as u16),
            DataSize::Word => self.bus.load16(addr),
        }
    }

    fn store(&mut self, addr: u16, size: DataSize, value: u16) -> Result<(), ()> {
        match size {
            DataSize::Byte => self.bus.store8(addr, value as u8),
            DataSize::Word => self.bus.store16(addr, value),
        }
    }

    // helper function for command with operation on register
    // B, C, D, E, H, L, (HL), A, d8
    fn get_r8(&self, target: &Target) -> Result<u8, ()> {
        match target {
            Target::B  => Ok(self.regs.b),
            Target::C  => Ok(self.regs.c),
            Target::D  => Ok(self.regs.d),
            Target::E  => Ok(self.regs.e),
            Target::H  => Ok(self.regs.h),
            Target::L  => Ok(self.regs.l),
            Target::HL => Ok(self.load(self.regs.get_hl(), DataSize::Byte)? as u8),
            Target::A  => Ok(self.regs.a),
            Target::D8 => Ok(self.load(self.pc, DataSize::Byte)? as u8),
            _ => {
                info!("Invalid target for instruction {:?}", target);
                return Err(());
            }
        }
    }

    fn set_r8(&mut self, target: &Target, value: u8) -> Result<(), ()> {
        match target {
            Target::A  => self.regs.a = value,
            Target::B  => self.regs.b = value,
            Target::C  => self.regs.c = value,
            Target::D  => self.regs.d = value,
            Target::E  => self.regs.e = value,
            Target::H  => self.regs.h = value,
            Target::HL => self.store(self.regs.get_hl(), DataSize::Byte, value as u16)?,
            Target::L  => self.regs.l = value,

            _ => {
                info!("Invalid target for instruction {:?}", target);
                        return Err(());
            }
        }
        Ok(())
    }

    fn check_condition(&self, condition: &Condition) -> bool {
        match condition {
            Condition::NotZero => !self.regs.f.zero,
            Condition::Zero => self.regs.f.zero,
            Condition::NotCarry => !self.regs.f.carry,
            Condition::Carry => self.regs.f.carry,
            Condition::Always => true,
        }
    }

    /// run single command in CPU return the clock length
    pub fn step(&mut self) -> Result<(), ()> {
        debug!("{}", self.dump());
        let clock = self.exec_one_instruction()?;
        self.bus.gpu.update(clock);
        self.bus.timer.update(clock);

        // handle interrupt
        if self.interrupt_state == InterruptState::IEnable ||
           self.interrupt_state == InterruptState::IDisableNext {
            let clock = self.handle_interrupt()?;

            self.bus.gpu.update(clock);
            self.bus.timer.update(clock);
        }

        // update interrupt state
        self.interrupt_state = match self.interrupt_state {
            InterruptState::IDisableNext => InterruptState::IDisable,
            InterruptState::IEnableNext => InterruptState::IEnable,
            _ => self.interrupt_state,
        };

        Ok(())
    }

    fn handle_interrupt(&mut self) -> Result<u64, ()> {
        // Vblank, priority 1, highest
        if self.bus.interruptenb.vblank && self.bus.gpu.is_interrupt {
            debug!("VBlank Interrupt");
            self.bus.gpu.is_interrupt = false;
            self.interrupt_state = InterruptState::IDisable;
            return self.execute(Instruction::RST(0x40))
        }
        Ok(0)
    }

    fn exec_one_instruction(&mut self) -> Result<u64, ()> {
        let byte = self.fetch()? as u8;
        if byte == 0xcb {
            let byte = self.fetch()? as u8;
            // CB instruction is full, should not fail
            let inst = CBInstruction::from_byte(byte);
            self.execute_cb(inst)
        } else {
            if let Some(inst) = Instruction::from_byte(byte) {
                self.execute(inst)
            } else {
                debug!("Unsupport instruction {:#x}", byte as u8);
                Err(())
            }
        }
    }

    // execute one non-prefix (0xcb) command, and return the clock passed
    fn execute(&mut self, inst: Instruction) -> Result<u64, ()> {
        let len = inst.len();
        let clock = inst.clock();
        match inst {
            Instruction::NOP => {},
            Instruction::JP(condition) => {
                if self.check_condition(&condition) {
                    let addr = self.load(self.pc, DataSize::Word)?;
                    self.pc = addr;
                    return Ok(16);
                }
            },
            Instruction::JPHL => {
                self.pc = self.regs.get_hl();
                return Ok(clock);
            }
            Instruction::DI => {
                self.interrupt_state = InterruptState::IDisableNext;
            }
            Instruction::EI => {
                self.interrupt_state = InterruptState::IEnableNext;
            }
            Instruction::LDIMM16(target) => {
                let imm = self.load(self.pc, DataSize::Word)?;
                match &target {
                    &Target::BC => self.regs.set_bc(imm),
                    &Target::DE => self.regs.set_de(imm),
                    &Target::HL => self.regs.set_hl(imm),
                    &Target::SP => self.sp = imm,
                    _ => {
                        info!("Invalid target for instruction {:?}", target);
                        return Err(());
                    }
                }
            }
            Instruction::LD16A => {
                let addr = self.load(self.pc, DataSize::Word)?;
                self.store(addr, DataSize::Byte, self.regs.a as u16)?;
            }
            Instruction::LDA16 => {
                let addr = self.load(self.pc, DataSize::Word)?;
                self.regs.a = self.load(addr, DataSize::Byte)? as u8;
            }
            Instruction::LDA16SP => {
                let addr = self.load(self.pc, DataSize::Word)?;
                self.store(addr, DataSize::Word, self.sp)?;
            }
            Instruction::LDSPHL => {
                self.sp = self.regs.get_hl();
            }
            Instruction::LDIMM8(target) => {
                let imm = self.load(self.pc, DataSize::Byte)? as u8;
                self.set_r8(&target, imm)?;
            }
            Instruction::LD8A => {
                let addr = 0xff00 + (self.load(self.pc, DataSize::Byte)?);
                self.store(addr, DataSize::Byte, self.regs.a as u16)?;
            }
            Instruction::LDA8 => {
                let addr = 0xff00 + (self.load(self.pc, DataSize::Byte)?);
                self.regs.a = self.load(addr, DataSize::Byte)? as u8;
            }
            Instruction::LDCA => {
                let addr = 0xff00 + self.regs.c as u16;
                self.store(addr, DataSize::Byte, self.regs.a as u16)?;
            }
            Instruction::LDAC => {
                let addr = 0xff00 + self.regs.c as u16;
                self.regs.a = self.load(addr, DataSize::Byte)? as u8;
            }
            Instruction::LDRR(source, target) => {
                match (&source, &target) {
                    (&Target::B,  &Target::B) => {},
                    (&Target::C,  &Target::B) => self.regs.b = self.regs.c,
                    (&Target::D,  &Target::B) => self.regs.b = self.regs.d,
                    (&Target::E,  &Target::B) => self.regs.b = self.regs.e,
                    (&Target::H,  &Target::B) => self.regs.b = self.regs.h,
                    (&Target::L,  &Target::B) => self.regs.b = self.regs.l,
                    (&Target::HL, &Target::B) => self.regs.b = self.load(self.regs.get_hl(), DataSize::Byte)? as u8,
                    (&Target::A,  &Target::B) => self.regs.b = self.regs.a,
                    (&Target::B,  &Target::C) => self.regs.c = self.regs.b,
                    (&Target::C,  &Target::C) => {},
                    (&Target::D,  &Target::C) => self.regs.c = self.regs.d,
                    (&Target::E,  &Target::C) => self.regs.c = self.regs.e,
                    (&Target::H,  &Target::C) => self.regs.c = self.regs.h,
                    (&Target::L,  &Target::C) => self.regs.c = self.regs.l,
                    (&Target::HL, &Target::C) => self.regs.c = self.load(self.regs.get_hl(), DataSize::Byte)? as u8,
                    (&Target::A,  &Target::C) => self.regs.c = self.regs.a,
                    (&Target::B,  &Target::D) => self.regs.d = self.regs.b,
                    (&Target::C,  &Target::D) => self.regs.d = self.regs.c,
                    (&Target::D,  &Target::D) => {},
                    (&Target::E,  &Target::D) => self.regs.d = self.regs.e,
                    (&Target::H,  &Target::D) => self.regs.d = self.regs.h,
                    (&Target::L,  &Target::D) => self.regs.d = self.regs.l,
                    (&Target::HL, &Target::D) => self.regs.d = self.load(self.regs.get_hl(), DataSize::Byte)? as u8,
                    (&Target::A,  &Target::D) => self.regs.d = self.regs.a,
                    (&Target::B,  &Target::E) => self.regs.e = self.regs.b,
                    (&Target::C,  &Target::E) => self.regs.e = self.regs.c,
                    (&Target::D,  &Target::E) => self.regs.e = self.regs.d,
                    (&Target::E,  &Target::E) => {},
                    (&Target::H,  &Target::E) => self.regs.e = self.regs.h,
                    (&Target::L,  &Target::E) => self.regs.e = self.regs.l,
                    (&Target::HL, &Target::E) => self.regs.e = self.load(self.regs.get_hl(), DataSize::Byte)? as u8,
                    (&Target::A,  &Target::E) => self.regs.e = self.regs.a,
                    (&Target::B,  &Target::H) => self.regs.h = self.regs.b,
                    (&Target::C,  &Target::H) => self.regs.h = self.regs.c,
                    (&Target::D,  &Target::H) => self.regs.h = self.regs.d,
                    (&Target::E,  &Target::H) => self.regs.h = self.regs.e,
                    (&Target::H,  &Target::H) => {},
                    (&Target::L,  &Target::H) => self.regs.h = self.regs.l,
                    (&Target::HL, &Target::H) => self.regs.h = self.load(self.regs.get_hl(), DataSize::Byte)? as u8,
                    (&Target::A,  &Target::H) => self.regs.h = self.regs.a,
                    (&Target::B,  &Target::L) => self.regs.l = self.regs.b,
                    (&Target::C,  &Target::L) => self.regs.l = self.regs.c,
                    (&Target::D,  &Target::L) => self.regs.l = self.regs.d,
                    (&Target::E,  &Target::L) => self.regs.l = self.regs.e,
                    (&Target::H,  &Target::L) => self.regs.l = self.regs.h,
                    (&Target::L,  &Target::L) => {},
                    (&Target::HL, &Target::L) => self.regs.l = self.load(self.regs.get_hl(), DataSize::Byte)? as u8,
                    (&Target::A,  &Target::L) => self.regs.l = self.regs.a,
                    (&Target::B,  &Target::HL) => self.store(self.regs.get_hl(), DataSize::Byte, self.regs.b as u16)?,
                    (&Target::C,  &Target::HL) => self.store(self.regs.get_hl(), DataSize::Byte, self.regs.c as u16)?,
                    (&Target::D,  &Target::HL) => self.store(self.regs.get_hl(), DataSize::Byte, self.regs.d as u16)?,
                    (&Target::E,  &Target::HL) => self.store(self.regs.get_hl(), DataSize::Byte, self.regs.e as u16)?,
                    (&Target::H,  &Target::HL) => self.store(self.regs.get_hl(), DataSize::Byte, self.regs.h as u16)?,
                    (&Target::L,  &Target::HL) => self.store(self.regs.get_hl(), DataSize::Byte, self.regs.l as u16)?,
                    (&Target::A,  &Target::HL) => self.store(self.regs.get_hl(), DataSize::Byte, self.regs.a as u16)?,
                    (&Target::B,  &Target::A) => self.regs.a = self.regs.b,
                    (&Target::C,  &Target::A) => self.regs.a = self.regs.c,
                    (&Target::D,  &Target::A) => self.regs.a = self.regs.d,
                    (&Target::E,  &Target::A) => self.regs.a = self.regs.e,
                    (&Target::H,  &Target::A) => self.regs.a = self.regs.h,
                    (&Target::L,  &Target::A) => self.regs.a = self.regs.l,
                    (&Target::HL, &Target::A) => self.regs.a = self.load(self.regs.get_hl(), DataSize::Byte)? as u8,
                    (&Target::A,  &Target::A) => {},
                    (&Target::A, &Target::BC) => self.store(self.regs.get_bc(), DataSize::Byte, self.regs.a as u16)?,
                    (&Target::A, &Target::DE) => self.store(self.regs.get_de(), DataSize::Byte, self.regs.a as u16)?,
                    (&Target::A, &Target::HLINC) => {
                        self.store(self.regs.get_hl(), DataSize::Byte, self.regs.a as u16)?;
                        self.regs.inc_hl();
                    },
                    (&Target::A, &Target::HLDEC) => {
                        self.store(self.regs.get_hl(), DataSize::Byte, self.regs.a as u16)?;
                        self.regs.dec_hl();
                    },
                    (&Target::BC, &Target::A) => {
                        self.regs.a = self.load(self.regs.get_bc(), DataSize::Byte)? as u8;
                    },
                    (&Target::DE, &Target::A) => {
                        self.regs.a = self.load(self.regs.get_de(), DataSize::Byte)? as u8;
                    },
                    (&Target::HLINC, &Target::A) => {
                        self.regs.a = self.load(self.regs.get_hl(), DataSize::Byte)? as u8;
                        self.regs.inc_hl();
                    },
                    (&Target::HLDEC, &Target::A) => {
                        self.regs.a = self.load(self.regs.get_hl(), DataSize::Byte)? as u8;
                        self.regs.dec_hl();
                    },
                    (_, _) => {
                        info!("Invalid target for instruction {:?} {:?}", source, target);
                        return Err(());
                    }
                }
            }
            Instruction::CALL(condition) => {
                if self.check_condition(&condition) {
                    let addr = self.load(self.pc, DataSize::Word)?;
                    self.store(self.sp-1, DataSize::Word, self.pc + 2)?;
                    self.sp -= 2;
                    self.pc = addr;
                    return Ok(24);
                }
            }
            Instruction::RET(condition) => {
                if self.check_condition(&condition) {
                    self.pc = self.load(self.sp + 1, DataSize::Word)?;
                    self.sp += 2;
                    let clock = if condition == Condition::Always { 16 } else { 20 };
                    return Ok(clock);
                }
            }
            Instruction::RETI => {
                self.interrupt_state = InterruptState::IEnable;
                self.pc = self.load(self.sp + 1, DataSize::Word)?;
                self.sp += 2;
                return Ok(clock);
            }
            Instruction::PUSH(target) => {
                let value = match target {
                    Target::BC => self.regs.get_bc(),
                    Target::DE => self.regs.get_de(),
                    Target::HL => self.regs.get_hl(),
                    Target::AF => self.regs.get_af(),
                    _ => {
                        info!("Invalid target for instruction {:?}", target);
                        return Err(());
                    }
                };
                self.store(self.sp-1, DataSize::Word, value)?;
                self.sp -= 2;
            }
            Instruction::POP(target) => {
                let value = self.load(self.sp+1, DataSize::Word)?;
                match target {
                    Target::BC => self.regs.set_bc(value),
                    Target::DE => self.regs.set_de(value),
                    Target::HL => self.regs.set_hl(value),
                    Target::AF => self.regs.set_af(value),
                    _ => {
                        info!("Invalid target for instruction {:?}", target);
                        return Err(());
                    }
                };
                self.sp += 2;
            }
            Instruction::JR(condition) => {
                if self.check_condition(&condition) {
                    let offset = self.load(self.pc, DataSize::Byte)? as i8;
                    self.pc = self.pc.wrapping_add(offset as u16);
                    self.pc += len;
                    return Ok(12);
                }
            }
            Instruction::INC16(target) => {
                match target {
                    Target::BC => self.regs.set_bc(self.regs.get_bc().wrapping_add(1)),
                    Target::DE => self.regs.set_de(self.regs.get_de().wrapping_add(1)),
                    Target::HL => self.regs.set_hl(self.regs.get_hl().wrapping_add(1)),
                    Target::SP => self.sp += 1,
                    _ => {
                        info!("Invalid target for instruction {:?}", target);
                        return Err(());
                    }
                }
            }
            Instruction::DEC16(target) => {
                match target {
                    Target::BC => self.regs.set_bc(self.regs.get_bc().wrapping_sub(1)),
                    Target::DE => self.regs.set_de(self.regs.get_de().wrapping_sub(1)),
                    Target::HL => self.regs.set_hl(self.regs.get_hl().wrapping_sub(1)),
                    Target::SP => self.sp -= 1,
                    _ => {
                        info!("Invalid target for instruction {:?}", target);
                        return Err(());
                    }
                }
            }
            Instruction::INC8(target) => {
                let mut value = self.get_r8(&target)?;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = (value & 0x0f) == 0x0f;
                value = value.wrapping_add(1);
                self.regs.f.zero = value == 0;
                // note that we have to update regs.a and sum after check other flag
                self.set_r8(&target, value)?;
            }
            Instruction::DEC8(target) => {
                let mut value = self.get_r8(&target)?;
                self.regs.f.subtract = true;
                self.regs.f.half_carry = (value & 0x0f) == 0x00;
                value = value.wrapping_sub(1);
                self.regs.f.zero = value == 0;
                // note that we have to update regs.a and sum after check other flag
                self.set_r8(&target, value)?;
            }
            Instruction::ADD(target) => {
                let value = self.get_r8(&target)?;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = (0x0f & self.regs.a) + (0x0f & value) > 0x0f;
                self.regs.f.carry = (self.regs.a as u16) + (value as u16) > 0xff;
                // note that we have to update regs.a and sum after check other flag
                self.regs.a = self.regs.a.wrapping_add(value);
                self.regs.f.zero = self.regs.a == 0;
            }
            Instruction::ADC(target) => {
                let value = self.get_r8(&target)?;
                let carry = if self.regs.f.carry { 1 } else { 0 };
                self.regs.f.subtract = false;
                self.regs.f.half_carry = (0x0f & self.regs.a) + (0x0f & value) + carry > 0x0f;
                self.regs.f.carry = (self.regs.a as u16) + (value as u16) + (carry as u16) > 0xff;
                // note that we have to update a after check flag
                self.regs.a = self.regs.a.wrapping_add(value).wrapping_add(carry);
                self.regs.f.zero = self.regs.a == 0;
            }
            Instruction::SUB(target) => {
                let value = self.get_r8(&target)?;
                self.regs.f.subtract = true;
                // FIXME: is half_carry and carry definition correct?
                self.regs.f.half_carry = (0x0f & self.regs.a) > (0x0f & value);
                self.regs.f.carry = self.regs.a > value;
                // note that we have to update regs.a and sum after check other flag
                self.regs.a = self.regs.a.wrapping_sub(value);
                self.regs.f.zero = self.regs.a == 0;
            }
            Instruction::SBC(target) => {
                let value = self.get_r8(&target)?;
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
                let value = self.get_r8(&target)?;
                self.regs.a &= value;
                self.regs.f.zero = self.regs.a == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = true;
                self.regs.f.carry = false;
            }
            Instruction::XOR(target) => {
                let value = self.get_r8(&target)?;
                self.regs.a ^= value;
                self.regs.f.zero = self.regs.a == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                self.regs.f.carry = false;
            }
            Instruction::OR(target) => {
                let value = self.get_r8(&target)?;
                self.regs.a |= value;
                self.regs.f.zero = self.regs.a == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                self.regs.f.carry = false;
            }
            Instruction::CMP(target) => {
                let value = self.get_r8(&target)?;
                self.regs.f.zero = self.regs.a == value;
                self.regs.f.subtract = true;
                self.regs.f.half_carry = (0x0f & self.regs.a) > (0x0f & value);
                self.regs.f.carry = self.regs.a < value;
            }
            Instruction::RST(addr) => {
                // note that PC is added in the fetch step
                // so RST will store PC+1, instead of PC.
                self.store(self.sp-1, DataSize::Word, self.pc)?;
                self.sp -= 2;
                self.pc = addr;
            }
            Instruction::CPL => {
                self.regs.a = !self.regs.a;
                self.regs.f.subtract = true;
                self.regs.f.half_carry = true;
            }
            Instruction::CCF => {
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                self.regs.f.carry = !self.regs.f.carry;
            }
            Instruction::ADDHL(target) => {
                let value = match &target {
                    &Target::BC => self.regs.get_bc(),
                    &Target::DE => self.regs.get_de(),
                    &Target::HL => self.regs.get_hl(),
                    &Target::SP => self.sp,
                    _ => {
                        info!("Invalid target for instruction {:?}", target);
                        return Err(());
                    }
                };
                let hl = self.regs.get_hl();
                self.regs.f.subtract = false;
                self.regs.f.half_carry = ((hl & 0xfff) + (value & 0xfff)) & 0x1000 != 0;
                self.regs.f.carry = (hl as u32) + (value as u32) > 0xffff;
                self.regs.set_hl(hl + value);
            }
            Instruction::RRA => {
                let value = self.regs.a;
                let result = (value >> 1) | ((self.regs.f.carry as u8) << 7);
                self.regs.f.zero = false;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                self.regs.f.carry = (value & 0x01) != 0;
                self.regs.a = result;
            }
            Instruction::DAA => {
                let mut value = self.regs.a as u16;
                // Please refer to Z80 manual
                // subtract
                if self.regs.f.subtract {
                    if self.regs.f.half_carry {
                        value = (value - 0x06) & 0xff;
                    }
                    if self.regs.f.carry {
                        value -= 0x60;
                    }
                } else {
                    if self.regs.f.half_carry || value & 0xf > 0 {
                        value += 0x06;
                    }
                    if self.regs.f.carry || value > 0x9F {
                        value += 0x60;
                    }
                }
                self.regs.f.zero = value == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                if value & 0x100 != 0 {
                    self.regs.f.carry = true;
                }
                self.regs.a = value as u8;
            }
            Instruction::RLCA => {
                // rotate target left
                let value = self.get_r8(&Target::A)?;
                let result = value << 1 | value >> 7;
                self.regs.f.zero = result == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                self.regs.f.carry = (value & 0x80) != 0;
                self.set_r8(&Target::A, result)?;
            }
        }
        self.pc += len;
        Ok(clock)
    }

    fn execute_cb(&mut self, inst: CBInstruction) -> Result<u64, ()> {
        let clock = inst.clock();
        match inst {
            CBInstruction::RLC(target) => {
                // rotate target left
                let value = self.get_r8(&target)?;
                let result = value << 1 | value >> 7;
                self.regs.f.zero = result == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                self.regs.f.carry = (value & 0x80) != 0;
                self.set_r8(&target, result)?;
            }
            CBInstruction::RRC(target) => {
                // rotate target right
                let value = self.get_r8(&target)?;
                let result = (value >> 1) | (value << 7);
                self.regs.f.zero = result == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                self.regs.f.carry = (value & 0x01) != 0;
                self.set_r8(&target, result)?;
            }
            CBInstruction::RL(target) => {
                // rotate target left through carry
                let value = self.get_r8(&target)?;
                let result = (value << 1) | (self.regs.f.carry as u8);
                self.regs.f.zero = result == 0;
                self.regs.f.half_carry = false;
                self.regs.f.subtract = false;
                self.regs.f.carry = (value & 0x80) != 0;
                self.set_r8(&target, result)?;
            }
            CBInstruction::RR(target) => {
                // rotate target right through carry
                let value = self.get_r8(&target)?;
                let result = (value >> 1) | ((self.regs.f.carry as u8) << 7);
                self.regs.f.zero = result == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                self.regs.f.carry = (value & 0x01) != 0;
                self.set_r8(&target, result)?;
            }
            CBInstruction::SLA(target) => {
                // shift target left into carry
                let value = self.get_r8(&target)?;
                let result = value << 1;
                self.regs.f.zero = result == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                self.regs.f.carry = (value & 0x80) != 0;
                self.set_r8(&target, result)?;
            }
            CBInstruction::SRA(target) => {
                // shift target right into carry, MSB not change
                let value = self.get_r8(&target)?;
                let result = (value >> 1) | (value & 0x80);
                self.regs.f.zero = result == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                self.regs.f.carry = (value & 0x01) != 0;
                self.set_r8(&target, result)?;
            }
            CBInstruction::SWAP(target) => {
                // swap register nibble
                let value = self.get_r8(&target)?;
                let result = (value << 4) | (value >> 4);
                self.regs.f.zero = result == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                self.regs.f.carry = false;
                self.set_r8(&target, result)?;
            }
            CBInstruction::SRL(target) => {
                // shift target right into carry, MSB to 0
                let value = self.get_r8(&target)?;
                let result = value >> 1;
                self.regs.f.zero = result == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = false;
                self.regs.f.carry = (value & 0x01) != 0;
                self.set_r8(&target, result)?;
            }
            CBInstruction::BIT(target, offset) => {
                // shift target right into carry, MSB to 0
                let value = (self.get_r8(&target)? >> offset) & 0x01;
                self.regs.f.zero = value == 0;
                self.regs.f.subtract = false;
                self.regs.f.half_carry = true;
            }
            CBInstruction::RES(target, offset) => {
                let value = self.get_r8(&target)?;
                self.set_r8(&target, value & !(1 << offset))?;
            }
            CBInstruction::SET(target, offset) => {
                let value = self.get_r8(&target)?;
                self.set_r8(&target, value | (1 << offset))?;
            }
        }
        Ok(clock)
    }

    pub fn dump(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("\tPC:{:04X} SP:{:04X}\t", self.pc, self.sp));
        output.push_str(&format!("{}\t", self.regs));
        let byte = self.load(self.pc, DataSize::Byte).unwrap() as u8;
        if byte == 0xcb {
            let byte = self.load(self.pc+1, DataSize::Byte).unwrap() as u8;
            output.push_str(&format!("byte:{:02X}\t", byte));
            output.push_str(&format!("inst:{:?}", CBInstruction::from_byte(byte)));
        } else {
            output.push_str(&format!("byte:{:02X}\t", byte));
            output.push_str(&format!("inst:{:?}", Instruction::from_byte(byte)));
        }
        output
    }
}
