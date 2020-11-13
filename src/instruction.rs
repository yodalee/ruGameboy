#[derive(Debug)]
pub enum Instruction {
    NOP,
    JP,
    DI,
    LDIMM16(Target),
    LD16A,
    LDA16,
    LDIMM8(Target),
    LD8A,
    LDA8,
    LDRR(Source, Target),
    CALL,
    RET(Condition),
    PUSH(Target),
    POP(Target),
    JR(Condition),
    INC(Target),
}

type Source = Target;
#[derive(Debug)]
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
}

#[derive(Debug)]
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
            0xc3 => Some(Instruction::JP),
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
            0xcd => Some(Instruction::CALL),
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
            0x2a => Some(Instruction::LDRR(Target::HLINC, Target::A)),
            0x3a => Some(Instruction::LDRR(Target::HLDEC, Target::A)),
            0x7d => Some(Instruction::LDRR(Target::L, Target::A)),
            0x7c => Some(Instruction::LDRR(Target::H, Target::A)),
            0x18 => Some(Instruction::JR(Condition::Always)),
            0x20 => Some(Instruction::JR(Condition::NotZero)),
            0x28 => Some(Instruction::JR(Condition::Zero)),
            0x30 => Some(Instruction::JR(Condition::NotCarry)),
            0x38 => Some(Instruction::JR(Condition::Carry)),
            0x03 => Some(Instruction::INC(Target::BC)),
            0x13 => Some(Instruction::INC(Target::DE)),
            0x23 => Some(Instruction::INC(Target::HL)),
            0x33 => Some(Instruction::INC(Target::SP)),
            _ => None
        }
    }

    pub fn len(&self) -> u16 {
        match self {
            Instruction::NOP => 1,
            Instruction::JP => 0,
            Instruction::DI => 1,
            Instruction::LDIMM16(_) => 3,
            Instruction::LD16A => 3,
            Instruction::LDA16 => 3,
            Instruction::LDIMM8(_) => 2,
            Instruction::LD8A => 2,
            Instruction::LDA8 => 2,
            Instruction::LDRR(_, _) => 1,
            Instruction::CALL => 0,
            Instruction::RET(_) => 1,
            Instruction::PUSH(_) => 1,
            Instruction::POP(_)  => 1,
            Instruction::JR(_) => 2,
            Instruction::INC(_) => 1,
        }
    }
}
