#[derive(Debug)]
pub enum Instruction {
    NOP,
    JP(Condition),
    DI,
    LDIMM16(Target),
    LDIMM8(Target),
    LD16A,
    LDA16,
    LD8A,
    LDA8,
    LDCA,
    LDAC,
    LDRR(Source, Target),
    CALL(Condition),
    RET(Condition),
    PUSH(Target),
    POP(Target),
    JR(Condition),
    INC16(Target),
    DEC16(Target),
    INC8(Target),
    DEC8(Target),
    ADD(Target),
    ADC(Target),
    SUB(Target),
    SBC(Target),
    AND(Target),
    XOR(Target),
    OR(Target),
    CMP(Target),
    RST(u16),
}

type Source = Target;
#[derive(Debug,PartialEq)]
pub enum Target {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    HLINC,
    HLDEC,
    SP,
    D8
}

#[derive(Debug,PartialEq)]
pub enum Condition {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

impl Instruction {
    pub fn from_byte(byte: u8) -> Option<Instruction> {
        match byte {
            0x00 => Some(Instruction::NOP),
            0xc2 => Some(Instruction::JP(Condition::NotZero)),
            0xc3 => Some(Instruction::JP(Condition::Always)),
            0xca => Some(Instruction::JP(Condition::Zero)),
            0xd2 => Some(Instruction::JP(Condition::NotCarry)),
            0xda => Some(Instruction::JP(Condition::Carry)),
            0xf3 => Some(Instruction::DI),
            0x01 => Some(Instruction::LDIMM16(Target::BC)),
            0x11 => Some(Instruction::LDIMM16(Target::DE)),
            0x21 => Some(Instruction::LDIMM16(Target::HL)),
            0x31 => Some(Instruction::LDIMM16(Target::SP)),
            0xea => Some(Instruction::LD16A),
            0xfa => Some(Instruction::LDA16),
            0x06 => Some(Instruction::LDIMM8(Target::B)),
            0x16 => Some(Instruction::LDIMM8(Target::D)),
            0x26 => Some(Instruction::LDIMM8(Target::H)),
            0x36 => Some(Instruction::LDIMM8(Target::HL)),
            0x0e => Some(Instruction::LDIMM8(Target::C)),
            0x1e => Some(Instruction::LDIMM8(Target::E)),
            0x2e => Some(Instruction::LDIMM8(Target::L)),
            0x3e => Some(Instruction::LDIMM8(Target::A)),
            0xe0 => Some(Instruction::LD8A),
            0xf0 => Some(Instruction::LDA8),
            0xc4 => Some(Instruction::CALL(Condition::NotZero)),
            0xcc => Some(Instruction::CALL(Condition::Zero)),
            0xcd => Some(Instruction::CALL(Condition::Always)),
            0xd4 => Some(Instruction::CALL(Condition::NotCarry)),
            0xdc => Some(Instruction::CALL(Condition::Carry)),
            0xc0 => Some(Instruction::RET(Condition::NotZero)),
            0xc8 => Some(Instruction::RET(Condition::Zero)),
            0xc9 => Some(Instruction::RET(Condition::Always)),
            0xd0 => Some(Instruction::RET(Condition::NotCarry)),
            0xd8 => Some(Instruction::RET(Condition::Carry)),
            0xc5 => Some(Instruction::PUSH(Target::BC)),
            0xd5 => Some(Instruction::PUSH(Target::DE)),
            0xe5 => Some(Instruction::PUSH(Target::HL)),
            0xf5 => Some(Instruction::PUSH(Target::AF)),
            0xc1 => Some(Instruction::POP(Target::BC)),
            0xd1 => Some(Instruction::POP(Target::DE)),
            0xe1 => Some(Instruction::POP(Target::HL)),
            0xf1 => Some(Instruction::POP(Target::AF)),
            0x02 => Some(Instruction::LDRR(Target::A, Target::BC)),
            0x12 => Some(Instruction::LDRR(Target::A, Target::DE)),
            0x22 => Some(Instruction::LDRR(Target::A, Target::HLINC)),
            0x32 => Some(Instruction::LDRR(Target::A, Target::HLDEC)),
            0x0a => Some(Instruction::LDRR(Target::BC, Target::A)),
            0x1a => Some(Instruction::LDRR(Target::DE, Target::A)),
            0x2a => Some(Instruction::LDRR(Target::HLINC, Target::A)),
            0x3a => Some(Instruction::LDRR(Target::HLDEC, Target::A)),
            0x78 => Some(Instruction::LDRR(Target::B, Target::A)),
            0x7d => Some(Instruction::LDRR(Target::L, Target::A)),
            0x7c => Some(Instruction::LDRR(Target::H, Target::A)),
            0x40 => Some(Instruction::LDRR(Target::B,  Target::B)),
            0x41 => Some(Instruction::LDRR(Target::C,  Target::B)),
            0x42 => Some(Instruction::LDRR(Target::D,  Target::B)),
            0x43 => Some(Instruction::LDRR(Target::E,  Target::B)),
            0x44 => Some(Instruction::LDRR(Target::H,  Target::B)),
            0x45 => Some(Instruction::LDRR(Target::L,  Target::B)),
            0x46 => Some(Instruction::LDRR(Target::HL, Target::B)),
            0x47 => Some(Instruction::LDRR(Target::A,  Target::B)),
            0x48 => Some(Instruction::LDRR(Target::B,  Target::C)),
            0x49 => Some(Instruction::LDRR(Target::C,  Target::C)),
            0x4a => Some(Instruction::LDRR(Target::D,  Target::C)),
            0x4b => Some(Instruction::LDRR(Target::E,  Target::C)),
            0x4c => Some(Instruction::LDRR(Target::H,  Target::C)),
            0x4d => Some(Instruction::LDRR(Target::L,  Target::C)),
            0x4e => Some(Instruction::LDRR(Target::HL, Target::C)),
            0x4f => Some(Instruction::LDRR(Target::A,  Target::C)),
            0x50 => Some(Instruction::LDRR(Target::B,  Target::D)),
            0x51 => Some(Instruction::LDRR(Target::C,  Target::D)),
            0x52 => Some(Instruction::LDRR(Target::D,  Target::D)),
            0x53 => Some(Instruction::LDRR(Target::E,  Target::D)),
            0x54 => Some(Instruction::LDRR(Target::H,  Target::D)),
            0x55 => Some(Instruction::LDRR(Target::L,  Target::D)),
            0x56 => Some(Instruction::LDRR(Target::HL, Target::D)),
            0x57 => Some(Instruction::LDRR(Target::A,  Target::D)),
            0x58 => Some(Instruction::LDRR(Target::B,  Target::E)),
            0x59 => Some(Instruction::LDRR(Target::C,  Target::E)),
            0x5a => Some(Instruction::LDRR(Target::D,  Target::E)),
            0x5b => Some(Instruction::LDRR(Target::E,  Target::E)),
            0x5c => Some(Instruction::LDRR(Target::H,  Target::E)),
            0x5d => Some(Instruction::LDRR(Target::L,  Target::E)),
            0x5e => Some(Instruction::LDRR(Target::HL, Target::E)),
            0x5f => Some(Instruction::LDRR(Target::A,  Target::E)),
            0x60 => Some(Instruction::LDRR(Target::B,  Target::H)),
            0x61 => Some(Instruction::LDRR(Target::C,  Target::H)),
            0x62 => Some(Instruction::LDRR(Target::D,  Target::H)),
            0x63 => Some(Instruction::LDRR(Target::E,  Target::H)),
            0x64 => Some(Instruction::LDRR(Target::H,  Target::H)),
            0x65 => Some(Instruction::LDRR(Target::L,  Target::H)),
            0x66 => Some(Instruction::LDRR(Target::HL, Target::H)),
            0x67 => Some(Instruction::LDRR(Target::A,  Target::H)),
            0x68 => Some(Instruction::LDRR(Target::B,  Target::L)),
            0x69 => Some(Instruction::LDRR(Target::C,  Target::L)),
            0x6a => Some(Instruction::LDRR(Target::D,  Target::L)),
            0x6b => Some(Instruction::LDRR(Target::E,  Target::L)),
            0x6c => Some(Instruction::LDRR(Target::H,  Target::L)),
            0x6d => Some(Instruction::LDRR(Target::L,  Target::L)),
            0x6e => Some(Instruction::LDRR(Target::HL, Target::L)),
            0x6f => Some(Instruction::LDRR(Target::A,  Target::L)),
            0x18 => Some(Instruction::JR(Condition::Always)),
            0x20 => Some(Instruction::JR(Condition::NotZero)),
            0x28 => Some(Instruction::JR(Condition::Zero)),
            0x30 => Some(Instruction::JR(Condition::NotCarry)),
            0x38 => Some(Instruction::JR(Condition::Carry)),
            0x03 => Some(Instruction::INC16(Target::BC)),
            0x13 => Some(Instruction::INC16(Target::DE)),
            0x23 => Some(Instruction::INC16(Target::HL)),
            0x33 => Some(Instruction::INC16(Target::SP)),
            0x0b => Some(Instruction::DEC16(Target::BC)),
            0x1b => Some(Instruction::DEC16(Target::DE)),
            0x2b => Some(Instruction::DEC16(Target::HL)),
            0x3b => Some(Instruction::DEC16(Target::SP)),
            0x04 => Some(Instruction::INC8(Target::B)),
            0x14 => Some(Instruction::INC8(Target::D)),
            0x24 => Some(Instruction::INC8(Target::H)),
            0x34 => Some(Instruction::INC8(Target::HL)),
            0x0c => Some(Instruction::INC8(Target::C)),
            0x1c => Some(Instruction::INC8(Target::E)),
            0x2c => Some(Instruction::INC8(Target::L)),
            0x3c => Some(Instruction::INC8(Target::A)),
            0x05 => Some(Instruction::DEC8(Target::B)),
            0x15 => Some(Instruction::DEC8(Target::D)),
            0x25 => Some(Instruction::DEC8(Target::H)),
            0x35 => Some(Instruction::DEC8(Target::HL)),
            0x0d => Some(Instruction::DEC8(Target::C)),
            0x1d => Some(Instruction::DEC8(Target::E)),
            0x2d => Some(Instruction::DEC8(Target::L)),
            0x3d => Some(Instruction::DEC8(Target::A)),
            0x80 => Some(Instruction::ADD(Target::B)),
            0x81 => Some(Instruction::ADD(Target::C)),
            0x82 => Some(Instruction::ADD(Target::D)),
            0x83 => Some(Instruction::ADD(Target::E)),
            0x84 => Some(Instruction::ADD(Target::H)),
            0x85 => Some(Instruction::ADD(Target::L)),
            0x86 => Some(Instruction::ADD(Target::HL)),
            0x87 => Some(Instruction::ADD(Target::A)),
            0xc6 => Some(Instruction::ADD(Target::D8)),
            0x88 => Some(Instruction::ADC(Target::B)),
            0x89 => Some(Instruction::ADC(Target::C)),
            0x8a => Some(Instruction::ADC(Target::D)),
            0x8b => Some(Instruction::ADC(Target::E)),
            0x8c => Some(Instruction::ADC(Target::H)),
            0x8d => Some(Instruction::ADC(Target::L)),
            0x8e => Some(Instruction::ADC(Target::HL)),
            0x8f => Some(Instruction::ADC(Target::A)),
            0xce => Some(Instruction::ADC(Target::D8)),
            0x90 => Some(Instruction::SUB(Target::B)),
            0x91 => Some(Instruction::SUB(Target::C)),
            0x92 => Some(Instruction::SUB(Target::D)),
            0x93 => Some(Instruction::SUB(Target::E)),
            0x94 => Some(Instruction::SUB(Target::H)),
            0x95 => Some(Instruction::SUB(Target::L)),
            0x96 => Some(Instruction::SUB(Target::HL)),
            0x97 => Some(Instruction::SUB(Target::A)),
            0xd6 => Some(Instruction::SUB(Target::D8)),
            0x98 => Some(Instruction::SBC(Target::B)),
            0x99 => Some(Instruction::SBC(Target::C)),
            0x9a => Some(Instruction::SBC(Target::D)),
            0x9b => Some(Instruction::SBC(Target::E)),
            0x9c => Some(Instruction::SBC(Target::H)),
            0x9d => Some(Instruction::SBC(Target::L)),
            0x9e => Some(Instruction::SBC(Target::HL)),
            0x9f => Some(Instruction::SBC(Target::A)),
            0xde => Some(Instruction::SBC(Target::D8)),
            0xa0 => Some(Instruction::AND(Target::B)),
            0xa1 => Some(Instruction::AND(Target::C)),
            0xa2 => Some(Instruction::AND(Target::D)),
            0xa3 => Some(Instruction::AND(Target::E)),
            0xa4 => Some(Instruction::AND(Target::H)),
            0xa5 => Some(Instruction::AND(Target::L)),
            0xa6 => Some(Instruction::AND(Target::HL)),
            0xa7 => Some(Instruction::AND(Target::A)),
            0xe6 => Some(Instruction::AND(Target::D8)),
            0xa8 => Some(Instruction::XOR(Target::B)),
            0xa9 => Some(Instruction::XOR(Target::C)),
            0xaa => Some(Instruction::XOR(Target::D)),
            0xab => Some(Instruction::XOR(Target::E)),
            0xac => Some(Instruction::XOR(Target::H)),
            0xad => Some(Instruction::XOR(Target::L)),
            0xae => Some(Instruction::XOR(Target::HL)),
            0xaf => Some(Instruction::XOR(Target::A)),
            0xee => Some(Instruction::XOR(Target::D8)),
            0xb0 => Some(Instruction::OR(Target::B)),
            0xb1 => Some(Instruction::OR(Target::C)),
            0xb2 => Some(Instruction::OR(Target::D)),
            0xb3 => Some(Instruction::OR(Target::E)),
            0xb4 => Some(Instruction::OR(Target::H)),
            0xb5 => Some(Instruction::OR(Target::L)),
            0xb6 => Some(Instruction::OR(Target::HL)),
            0xb7 => Some(Instruction::OR(Target::A)),
            0xf6 => Some(Instruction::OR(Target::D8)),
            0xb8 => Some(Instruction::CMP(Target::B)),
            0xb9 => Some(Instruction::CMP(Target::C)),
            0xba => Some(Instruction::CMP(Target::D)),
            0xbb => Some(Instruction::CMP(Target::E)),
            0xbc => Some(Instruction::CMP(Target::H)),
            0xbd => Some(Instruction::CMP(Target::L)),
            0xbe => Some(Instruction::CMP(Target::HL)),
            0xbf => Some(Instruction::CMP(Target::A)),
            0xfe => Some(Instruction::CMP(Target::D8)),
            0xc7 => Some(Instruction::RST(0x00)),
            0xcf => Some(Instruction::RST(0x08)),
            0xd7 => Some(Instruction::RST(0x10)),
            0xdf => Some(Instruction::RST(0x18)),
            0xe7 => Some(Instruction::RST(0x20)),
            0xef => Some(Instruction::RST(0x28)),
            0xf7 => Some(Instruction::RST(0x30)),
            0xff => Some(Instruction::RST(0x38)),
            0xe2 => Some(Instruction::LDCA),
            0xf2 => Some(Instruction::LDAC),
            _ => None
        }
    }

    pub fn len(&self) -> u16 {
        match self {
            Instruction::NOP => 1,
            Instruction::JP(_) => 3,
            Instruction::DI => 1,
            Instruction::LDIMM16(_) => 3,
            Instruction::LDIMM8(_) => 2,
            Instruction::LD16A => 3,
            Instruction::LDA16 => 3,
            Instruction::LD8A => 2,
            Instruction::LDA8 => 2,
            Instruction::LDCA => 2,
            Instruction::LDAC => 2,
            Instruction::LDRR(_, _) => 1,
            Instruction::CALL(_) => 3,
            Instruction::RET(_) => 1,
            Instruction::PUSH(_) => 1,
            Instruction::POP(_)  => 1,
            Instruction::JR(_) => 2,
            Instruction::INC16(_) => 1,
            Instruction::DEC16(_) => 1,
            Instruction::INC8(_) => 1,
            Instruction::DEC8(_) => 1,
            Instruction::ADD(t) => if t == &Target::D8 { 2 } else { 1 },
            Instruction::ADC(t) => if t == &Target::D8 { 2 } else { 1 },
            Instruction::SUB(t) => if t == &Target::D8 { 2 } else { 1 },
            Instruction::SBC(t) => if t == &Target::D8 { 2 } else { 1 },
            Instruction::AND(t) => if t == &Target::D8 { 2 } else { 1 },
            Instruction::XOR(t) => if t == &Target::D8 { 2 } else { 1 },
            Instruction::OR(t) =>  if t == &Target::D8 { 2 } else { 1 },
            Instruction::CMP(t) => if t == &Target::D8 { 2 } else { 1 },
            Instruction::RST(_) => 0,
        }
    }

    pub fn clock(&self) -> u32 {
        // return clock of instruction, default non-taken action
        match self {
            Instruction::NOP => 4,
            Instruction::JP(_) => 12,
            Instruction::DI => 4,
            Instruction::LDIMM16(_) => 12,
            Instruction::LDIMM8(t) => if t == &Target::HL { 12 } else { 8 },
            Instruction::LD16A => 16,
            Instruction::LDA16 => 16,
            Instruction::LD8A => 12,
            Instruction::LDA8 => 12,
            Instruction::LDCA => 8,
            Instruction::LDAC => 8,
            Instruction::LDRR(s, t) =>
                if s == &Target::HL || t == &Target::HL {
                    8
                } else {
                    4
                },
            Instruction::CALL(_) => 12,
            Instruction::RET(_) => 8,
            Instruction::PUSH(_) => 16,
            Instruction::POP(_)  => 12,
            Instruction::JR(_) => 8,
            Instruction::INC16(_) => 8,
            Instruction::DEC16(_) => 8,
            Instruction::INC8(t) | Instruction::DEC8(t) =>
                if t == &Target::HL {
                    12
                } else {
                    4
                },
            Instruction::ADD(t) => if t == &Target::D8 || t == &Target::HL { 2 } else { 1 },
            Instruction::ADC(t) => if t == &Target::D8 || t == &Target::HL { 2 } else { 1 },
            Instruction::SUB(t) => if t == &Target::D8 || t == &Target::HL { 2 } else { 1 },
            Instruction::SBC(t) => if t == &Target::D8 || t == &Target::HL { 2 } else { 1 },
            Instruction::AND(t) => if t == &Target::D8 || t == &Target::HL { 2 } else { 1 },
            Instruction::XOR(t) => if t == &Target::D8 || t == &Target::HL { 2 } else { 1 },
            Instruction::OR(t) =>  if t == &Target::D8 || t == &Target::HL { 2 } else { 1 },
            Instruction::CMP(t) => if t == &Target::D8 || t == &Target::HL { 2 } else { 1 },
            Instruction::RST(_) => 16,
        }
    }
}
