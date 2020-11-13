#[derive(Debug,Default)]
struct FlagRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool
}

const ZERO_FLAG_SHIFT: u8 = 7;
const SUBTRACT_FLAG_SHIFT: u8 = 6;
const HALFCARRY_FLAG_SHIFT: u8 = 5;
const CRARRY_FLAG_SHIFT: u8 = 4;

impl std::convert::From<FlagRegister> for u8 {
    fn from(flag: FlagRegister) -> u8 {
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
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: FlagRegister,
    h: u8,
    l: u8,
}

impl Register {
    fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    fn set_bc(&mut self, value: u16) {
        self.b = ((value >> 8) & 0xff) as u8;
        self.c = (value & 0xff) as u8;
    }

    fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    fn set_de(&mut self, value: u16) {
        self.d = ((value >> 8) & 0xff) as u8;
        self.e = (value & 0xff) as u8;
    }

    fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    fn set_hl(&mut self, value: u16) {
        self.h = ((value >> 8) & 0xff) as u8;
        self.l = (value & 0xff) as u8;
    }
}
