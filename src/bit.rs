#![allow(unused)]

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
        if formatter.alternate() {
            write!(formatter, "Bit({})", *self as i32)
        } else {
            write!(formatter, "{}", *self as i32)
        }
    }
}
