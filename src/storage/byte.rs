use crate::{bit, storage::word};
use std::ops::BitAnd;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Byte {
    inner: u8,
}

// public items
impl Byte {
    pub const ZEROED: Byte = Byte { inner: 0x00 };
    pub const FILLED: Byte = Byte { inner: 0xFF };

    #[must_use]
    pub fn new(words: [word::Word; 2]) -> Byte {
        let [a, b, c, d] = words[0].into_bits();
        let [e, f, g, h] = words[1].into_bits();
        Byte::from_bits([a, b, c, d, e, f, g, h])
    }

    #[must_use]
    pub fn into_words(self) -> [word::Word; 2] {
        let [a, b, c, d, e, f, g, h] = self.into_bits();
        [word::Word::new([a, b, c, d]), word::Word::new([e, f, g, h])]
    }

    #[must_use]
    pub fn get_first(self) -> word::Word {
        let [a, b, c, d, ..] = self.into_bits();
        word::Word::new([a, b, c, d])
    }

    #[must_use]
    pub fn get_second(self) -> word::Word {
        let [.., a, b, c, d] = self.into_bits();
        word::Word::new([a, b, c, d])
    }

    /// Will return `None` if `ith_word` is greater than or equal to 2
    #[must_use]
    pub fn get_word(self, ith_word: usize) -> Option<word::Word> {
        match ith_word {
            0 => Some(self.get_first()),
            1 => Some(self.get_second()),
            _ => None,
        }
    }

    pub fn set_first(&mut self, word: word::Word) -> word::Word {
        let [first, second] = self.into_words();
        *self = Byte::new([word, second]);
        first
    }

    pub fn set_second(&mut self, word: word::Word) -> word::Word {
        let [first, second] = self.into_words();
        *self = Byte::new([first, word]);
        second
    }

    /// Will return `None` if `ith_word` is greater than or equal to 2
    pub fn set_word(&mut self, word: word::Word, ith_word: usize) -> Option<word::Word> {
        match ith_word {
            0 => Some(self.set_first(word)),
            1 => Some(self.set_second(word)),
            _ => None,
        }
    }
}

// private items
impl Byte {
    #[must_use]
    fn from_bits(bits: [bit::Bit; 8]) -> Byte {
        Byte {
            inner: bits.into_iter().fold(0, |x, bit| (x << 1) + bit.into_u8()),
        }
    }

    #[must_use]
    fn into_bits(self) -> [bit::Bit; 8] {
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
    #[must_use]
    fn get_bit(self, ith_bit: u8) -> Option<bit::Bit> {
        match ith_bit {
            ..8 => bit::Bit::try_from_u8(self.inner.unbounded_shr(7 - ith_bit as u32).bitand(1)),
            _ => None,
        }
    }
}

impl std::fmt::Debug for Byte {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match formatter.alternate() {
            false => write!(formatter, "{:2X}", self.inner),
            true => write!(formatter, "Byte(0x{:2X})", self.inner),
        }
    }
}
