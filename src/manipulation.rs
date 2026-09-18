use crate::bit;
use crate::storage;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UInt {
    inner: [bit::Bit; 4],
}

impl UInt {
    pub const ZERO: UInt = UInt {
        inner: [bit::Zero; 4],
    };

    pub fn from_bits(bits: [bit::Bit; 4]) -> UInt {
        UInt { inner: bits }
    }

    pub fn from_word(word: storage::Word) -> UInt {
        UInt::from_bits(word.into_bits())
    }

    #[cfg(test)]
    pub fn from_u8(u8: u8) -> UInt {
        UInt::from_word(storage::Word::from_u8(u8))
    }

    pub fn into_bits(self) -> [bit::Bit; 4] {
        self.inner
    }

    pub fn into_word(self) -> storage::Word {
        storage::Word::new(self.into_bits())
    }

    pub fn increment(&mut self) -> Overflow {
        let mut overflow: bit::Bit = bit::One;

        for idx in (0usize..4).rev() {
            [self.inner[idx], overflow] =
                [self.inner[idx].xor(overflow), self.inner[idx].and(overflow)]
        }

        Overflow(overflow)
    }

    pub fn decrement(&mut self) -> Underflow {
        let mut underflow: bit::Bit = bit::One;

        for idx in (0usize..4).rev() {
            [self.inner[idx], underflow] = [
                self.inner[idx].xor(underflow),
                self.inner[idx].not().and(underflow),
            ]
        }

        Underflow(underflow)
    }

    pub fn cmp(self, rhs: UInt) -> std::cmp::Ordering {
        use std::cmp::Ordering::Equal;

        for idx in 0..4 {
            match self.inner[idx].cmp(rhs.inner[idx]) {
                Equal => continue,
                other => return other,
            }
        }

        Equal
    }

    pub fn add(self, rhs: UInt) -> (UInt, Overflow) {
        todo!("implement `UInt::add`")
    }

    pub fn sub(self, rhs: UInt) -> (UInt, Overflow) {
        todo!("implement `UInt::sub`")
    }

    pub fn mul(self, rhs: UInt) -> (UInt, Overflow) {
        todo!("implement `UInt::mul`")
    }

    pub fn div_rem(self, rhs: UInt) -> (UInt, UInt, Overflow) {
        todo!("implement `UInt::add`")
    }
}

impl std::fmt::Debug for UInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.into_word().fmt(f)
    }
}

pub struct Overflow(pub bit::Bit);
pub struct Underflow(pub bit::Bit);

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn word_conversion() {
        use bit::{One as O, Zero as Z};

        fn iteration(bits: [bit::Bit; 4], u8: u8) {
            let word = storage::Word::new(bits);
            let uint = UInt::from_bits(bits);

            assert_eq!(word, uint.into_word());
            assert_eq!(UInt::from_word(word), uint);
            assert_eq!(UInt::from_word(uint.into_word()), uint);

            let from_u8 = UInt::from_u8(u8);

            assert_eq!(uint, from_u8);
        }

        iteration([Z, Z, Z, Z], 0);
        iteration([Z, Z, Z, O], 1);
        iteration([Z, Z, O, Z], 2);
        iteration([Z, Z, O, O], 3);
        iteration([Z, O, Z, Z], 4);
        iteration([Z, O, Z, O], 5);
        iteration([Z, O, O, Z], 6);
        iteration([Z, O, O, O], 7);
        iteration([O, Z, Z, Z], 8);
        iteration([O, Z, Z, O], 9);
        iteration([O, Z, O, Z], 10);
        iteration([O, Z, O, O], 11);
        iteration([O, O, Z, Z], 12);
        iteration([O, O, Z, O], 13);
        iteration([O, O, O, Z], 14);
        iteration([O, O, O, O], 15);
    }

    #[test]
    fn increment() {
        for i in 0..15 {
            let mut less = UInt::from_u8(i);
            let more = UInt::from_u8(i + 1);
            let overflow = less.increment();
            assert_eq!(less, more);
            assert_eq!(overflow.0, bit::Zero);
        }

        let mut max = UInt::from_u8(15);
        let min = UInt::from_u8(0);
        let overflow = max.increment();
        assert_eq!(max, min);
        assert_eq!(overflow.0, bit::One);
    }

    #[test]
    fn decrement() {
        for i in 0..15 {
            let less = UInt::from_u8(i);
            let mut more = UInt::from_u8(i + 1);
            let underflow = more.decrement();
            assert_eq!(less, more);
            assert_eq!(underflow.0, bit::Zero);
        }

        let max = UInt::from_u8(15);
        let mut min = UInt::from_u8(0);
        let underflow = min.decrement();
        assert_eq!(max, min);
        assert_eq!(underflow.0, bit::One);
    }
}
