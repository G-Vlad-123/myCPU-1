#![allow(unused)]

#[cfg(test)]
pub const ITER_BIT: [Bit; 2] = [Zero, One];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Bit {
    Zero = 0,
    One = 1,
}

// rust-analyzer(?) gave me an error of an unresolved import without `self::`
// don't ask me why
pub use self::Bit::{One, Zero};

impl Bit {
    pub fn from_bool(bool: bool) -> Bit {
        match bool {
            true => One,
            false => Zero,
        }
    }

    pub fn try_from_u8(u8: u8) -> Option<Bit> {
        match u8 {
            0 => Some(Zero),
            1 => Some(One),
            _ => None,
        }
    }

    pub fn into_u8(self) -> u8 {
        self.is_one() as u8
    }

    pub fn is_one(self) -> bool {
        self == One
    }

    pub fn is_zero(self) -> bool {
        self == Zero
    }

    pub fn not(self) -> Bit {
        match self {
            Zero => One,
            One => Zero,
        }
    }

    pub fn and(self, rhs: Bit) -> Bit {
        match (self, rhs) {
            (One, One) => One,
            _ => Zero,
        }
    }

    pub fn or(self, rhs: Bit) -> Bit {
        match (self, rhs) {
            (Zero, Zero) => Zero,
            _ => One,
        }
    }

    pub fn xor(self, rhs: Bit) -> Bit {
        Bit::from_bool(self != rhs)
    }
}

impl std::fmt::Debug for Bit {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match formatter.alternate() {
            false => write!(formatter, "{}", *self as i32),
            true => write!(formatter, "Bit({})", *self as i32),
        }
    }
}
