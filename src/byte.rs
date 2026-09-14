use std::ops::BitAnd;

use crate::bit::{self, Bit};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Word {
    inner: u8,
}

impl Word {
    pub fn new(bits: [bit::Bit; 4]) -> Word {
        Word {
            inner: bits.into_iter().fold(0, |x, bit| (x << 1) + bit.into_u8()),
        }
    }

    pub fn zeroed() -> Word {
        Word { inner: 0 }
    }

    pub fn into_bits(self) -> [bit::Bit; 4] {
        // SAFETY: `ith_bit` is always lesser than 4
        unsafe {
            [
                self.get_bit(0).unwrap_unchecked(),
                self.get_bit(1).unwrap_unchecked(),
                self.get_bit(2).unwrap_unchecked(),
                self.get_bit(3).unwrap_unchecked(),
            ]
        }
    }

    /// Will return `None` if and only if `ith_bit` is greater than or equal to 4
    pub fn get_bit(self, ith_bit: u8) -> Option<bit::Bit> {
        match ith_bit {
            ..4 => Bit::try_from_u8(self.inner.unbounded_shr(3 - ith_bit as u32).bitand(1)),
            _ => None,
        }
    }

    pub fn increment(self) -> Word {
        todo!()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Byte {
    inner: u8,
}

impl Byte {
    pub fn new(bits: [bit::Bit; 8]) -> Byte {
        Byte {
            inner: bits.into_iter().fold(0, |x, bit| (x << 1) + bit.into_u8()),
        }
    }

    pub fn from_words(first: Word, second: Word) -> Byte {
        let [a, b, c, d] = first.into_bits();
        let [e, f, g, h] = second.into_bits();
        Byte::new([a, b, c, d, e, f, g, h])
    }

    pub fn zeroed() -> Byte {
        Byte { inner: 0 }
    }

    pub fn into_words(self) -> [Word; 2] {
        let [a, b, c, d, e, f, g, h] = self.into_bits();
        [Word::new([a, b, c, d]), Word::new([e, f, g, h])]
    }

    pub fn into_bits(self) -> [bit::Bit; 8] {
        // SAFETY: `ith_bit` is always lesser than 8
        unsafe {
            [
                self.get_bit(0).unwrap_unchecked(),
                self.get_bit(1).unwrap_unchecked(),
                self.get_bit(2).unwrap_unchecked(),
                self.get_bit(3).unwrap_unchecked(),
                self.get_bit(4).unwrap_unchecked(),
                self.get_bit(5).unwrap_unchecked(),
                self.get_bit(6).unwrap_unchecked(),
                self.get_bit(7).unwrap_unchecked(),
            ]
        }
    }

    /// Will return `None` if and only if `ith_bit` is greater than or equal to 8
    pub fn get_bit(self, ith_bit: u8) -> Option<bit::Bit> {
        match ith_bit {
            ..8 => Bit::try_from_u8(self.inner.unbounded_shr(7 - ith_bit as u32).bitand(1)),
            _ => None,
        }
    }
}
