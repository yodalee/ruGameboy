use std::fmt;

#[derive(Debug,Default)]
pub struct FlagRegister {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool
}

const ZERO_FLAG_SHIFT: u8 = 7;
const SUBTRACT_FLAG_SHIFT: u8 = 6;
const HALFCARRY_FLAG_SHIFT: u8 = 5;
const CRARRY_FLAG_SHIFT: u8 = 4;

impl std::convert::From<&FlagRegister> for u8 {
    fn from(flag: &FlagRegister) -> u8 {
        ( if flag.zero { 1 << ZERO_FLAG_SHIFT } else { 0 } ) |
        ( if flag.subtract { 1 << SUBTRACT_FLAG_SHIFT } else { 0 } ) |
        ( if flag.half_carry { 1 << HALFCARRY_FLAG_SHIFT } else { 0 } ) |
        ( if flag.carry { 1 << CRARRY_FLAG_SHIFT } else { 0 } )
    }
}

impl std::convert::From<u8> for FlagRegister {
    fn from(byte: u8) -> FlagRegister {
        FlagRegister {
            zero: ((byte >> ZERO_FLAG_SHIFT) & 0b1) != 0,
            subtract: ((byte >> SUBTRACT_FLAG_SHIFT) & 0b1) != 0,
            half_carry: ((byte >> HALFCARRY_FLAG_SHIFT) & 0b1) != 0,
            carry: ((byte >> CRARRY_FLAG_SHIFT) & 0b1) != 0,
        }
    }
}

#[derive(Debug,Default)]
pub struct Register {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: FlagRegister,
    pub h: u8,
    pub l: u8,
}

impl Register {
    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | u8::from(&self.f) as u16
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = ((value >> 8) & 0xff) as u8;
        self.f = FlagRegister::from((value & 0xff) as u8);
    }

    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value >> 8) & 0xff) as u8;
        self.c = (value & 0xff) as u8;
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = ((value >> 8) & 0xff) as u8;
        self.e = (value & 0xff) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value >> 8) & 0xff) as u8;
        self.l = (value & 0xff) as u8;
    }

    //TODO, optimize this
    pub fn inc_hl(&mut self) {
        self.set_hl(self.get_hl().wrapping_add(1));
    }

    pub fn dec_hl(&mut self) {
        self.set_hl(self.get_hl().wrapping_sub(1));
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        output = format!("{} AF:{:02X}{:02X}", output, self.a, u8::from(&self.f));
        output = format!("{} BC:{:02X}{:02X}", output, self.b, self.c);
        output = format!("{} DE:{:02X}{:02X}", output, self.d, self.e);
        output = format!("{} HL:{:02x}{:02X}", output, self.h, self.l);
        write!(f, "{}", output)
    }
}

impl fmt::Display for FlagRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Z {} SUB {} HC {} C {}",
                    if self.zero { 1 } else { 0 },
                    if self.subtract { 1 } else { 0 },
                    if self.half_carry { 1 } else { 0 },
                    if self.carry { 1 } else { 0 })
    }
}
