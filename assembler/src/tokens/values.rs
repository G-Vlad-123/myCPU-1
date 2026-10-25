pub trait AppendTo {
    fn append_to(self, data: &mut Vec<Word>);
}

impl<T: Into<Word>> AppendTo for T {
    fn append_to(self, data: &mut Vec<Word>) {
        data.push(self.into())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Word {
    W0 = 0x0,
    W1 = 0x1,
    W2 = 0x2,
    W3 = 0x3,
    W4 = 0x4,
    W5 = 0x5,
    W6 = 0x6,
    W7 = 0x7,
    W8 = 0x8,
    W9 = 0x9,
    WA = 0xA,
    WB = 0xB,
    WC = 0xC,
    WD = 0xD,
    WE = 0xE,
    WF = 0xF,
}

impl From<[bool; 4]> for Word {
    fn from(array: [bool; 4]) -> Self {
        let int: u8 = ((array[0] as u8) << 3)
            | ((array[1] as u8) << 2)
            | ((array[2] as u8) << 1)
            | ((array[3] as u8) << 0);

        unsafe {
            // SAFETY: int will only be 4 bits long which covers all possible word values
            std::mem::transmute(int)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    RA,
    RB,
    RC,
    RD,
    RE,
}

impl Register {
    fn new(word: Word) -> Option<Register> {
        match word {
            Word::W0 => Some(Register::R0),
            Word::W1 => Some(Register::R1),
            Word::W2 => Some(Register::R2),
            Word::W3 => Some(Register::R3),
            Word::W4 => Some(Register::R4),
            Word::W5 => Some(Register::R5),
            Word::W6 => Some(Register::R6),
            Word::W7 => Some(Register::R7),
            Word::W8 => Some(Register::R8),
            Word::W9 => Some(Register::R9),
            Word::WA => Some(Register::RA),
            Word::WB => Some(Register::RB),
            Word::WC => Some(Register::RC),
            Word::WD => Some(Register::RD),
            Word::WE => Some(Register::RE),
            Word::WF => None,
        }
    }
}

impl TryFrom<Word> for Register {
    type Error = ();

    fn try_from(word: Word) -> Result<Self, Self::Error> {
        Self::new(word).ok_or(())
    }
}

impl From<Register> for Word {
    fn from(register: Register) -> Self {
        match register {
            Register::R0 => Word::W0,
            Register::R1 => Word::W1,
            Register::R2 => Word::W2,
            Register::R3 => Word::W3,
            Register::R4 => Word::W4,
            Register::R5 => Word::W5,
            Register::R6 => Word::W6,
            Register::R7 => Word::W7,
            Register::R8 => Word::W8,
            Register::R9 => Word::W9,
            Register::RA => Word::WA,
            Register::RB => Word::WB,
            Register::RC => Word::WC,
            Register::RD => Word::WD,
            Register::RE => Word::WE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Literal(pub Word);

impl From<Word> for Literal {
    fn from(word: Word) -> Self {
        Self(word)
    }
}

impl From<Literal> for Word {
    fn from(literal: Literal) -> Self {
        literal.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Register(Register),
    Literal(Literal),
}

impl From<Register> for Value {
    fn from(register: Register) -> Self {
        Self::Register(register)
    }
}

impl From<Literal> for Value {
    fn from(literal: Literal) -> Self {
        Self::Literal(literal)
    }
}

impl AppendTo for Value {
    fn append_to(self, data: &mut Vec<Word>) {
        match self {
            Self::Literal(literal) => {
                data.push(Word::WF);
                literal.append_to(data);
            }
            Self::Register(register) => register.append_to(data),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Address {
    pub high: Word,
    pub low: Word,
}

impl AppendTo for Address {
    fn append_to(self, data: &mut Vec<Word>) {
        self.high.append_to(data);
        self.low.append_to(data);
    }
}
