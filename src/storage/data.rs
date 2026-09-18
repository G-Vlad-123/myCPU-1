use crate::manipulation as manip;
use crate::storage::{byte::Byte, word::Word};

pub struct Data {
    inner: Box<[Byte]>,
}

struct Idx {
    byte: usize,

    /// Guaranteed to be eather 0 or 1
    word: usize,
}

impl Idx {
    fn compute(idx: Address) -> Idx {
        let idx =
            crate::storage::word::pair_to_u8(idx.high.into_word(), idx.low.into_word()) as usize;

        Idx {
            byte: idx / 2,
            word: idx % 2,
        }
    }
}

impl Data {
    pub const WORD_COUNT: usize = 512;

    #[must_use]
    pub fn new() -> Data {
        Data {
            inner: Box::from_iter(std::iter::repeat_n(
                Byte::ZEROED,
                Data::WORD_COUNT.div_ceil(2),
            )),
        }
    }

    #[must_use]
    #[cfg(test)]
    pub fn from_slice(slice: &[u8]) -> Data {
        use std::iter;
        use std::ops::Sub;

        Data {
            inner: Box::from_iter(iter::chain(
                slice.iter().copied().map(Byte::from_u8),
                iter::repeat_n(Byte::ZEROED, Data::WORD_COUNT.sub(slice.len()).div_ceil(2)),
            )),
        }
    }

    #[must_use]
    pub fn get_word(&self, addr: Address) -> Word {
        let idx = Idx::compute(addr);

        self.inner[idx.byte]
            .get_word(idx.word)
            .expect("`idx.word` is guaranteed to be 0 or 1")
    }

    pub fn set_word(&mut self, addr: Address, new: Word) -> Word {
        let idx = Idx::compute(addr);

        self.inner[idx.byte]
            .set_word(new, idx.word)
            .expect("`idx.word` is guaranteed to be 0 or 1")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Address {
    pub high: manip::UInt,
    pub low: manip::UInt,
}

impl Address {
    pub const NULL: Address = Address {
        high: manip::UInt::ZERO,
        low: manip::UInt::ZERO,
    };

    pub fn new(high: Word, low: Word) -> Address {
        Address {
            high: manip::UInt::from_word(high),
            low: manip::UInt::from_word(low),
        }
    }

    pub fn increment(&mut self) -> manip::Overflow {
        if self.low.increment().0.is_one() {
            self.high.increment()
        } else {
            manip::Overflow(crate::bit::Zero)
        }
    }

    pub fn offset(mut self, offset: u8) -> Address {
        for _ in 0..offset {
            self.increment();
        }

        self
    }

    pub fn jump(&mut self, high: Word, low: Word) {
        self.high = manip::UInt::from_word(high);
        self.low = manip::UInt::from_word(low);
    }
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn from_slice() {
        let data = Data::from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        for (idx, elem) in data.inner.iter().enumerate() {
            assert_eq!(elem.into_u8() as usize, if idx < 10 { idx } else { 0 });
        }
    }

    #[test]
    fn get_word() {
        use std::ops::Mul;

        let slice = &[0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10];
        let data = Data::from_slice(slice);

        let mut addr = Address::NULL;
        let mut at = 0xF;

        for _ in 0..slice.len().mul(2) {
            let word = data.get_word(addr);
            assert_eq!(word.into_u8(), at);
            addr.increment();
            at = at.wrapping_sub(1);
        }
    }
}
