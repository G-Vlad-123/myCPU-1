use crate::bit;
use std::ops::BitAnd;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Word {
    inner: u8,
}

impl Word {
    pub const ZEROED: Word = Word { inner: 0x00 };
    pub const FILLED: Word = Word { inner: 0x0F };

    #[must_use]
    pub fn new(bits: [bit::Bit; 4]) -> Word {
        Word {
            inner: bits.into_iter().fold(0, |x, bit| (x << 1) + bit.into_u8()),
        }
    }

    #[must_use]
    #[cfg(test)]
    /// Gets the least significant 4 bits of a u8
    pub fn from_u8(u8: u8) -> Word {
        Word { inner: u8 }
    }

    #[must_use]
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

    #[must_use]
    pub fn into_uint(self) -> crate::manipulation::UInt {
        crate::manipulation::UInt::from_word(self)
    }

    #[must_use]
    // #[cfg(test)]
    /// Gets the least significant 4 bits of a u8
    pub fn into_u8(self) -> u8 {
        self.inner
    }

    /// Will return `None` if and only if `ith_bit` is greater than or equal to 4
    #[must_use]
    pub fn get_bit(self, ith_bit: u8) -> Option<bit::Bit> {
        match ith_bit {
            ..4 => bit::Bit::try_from_u8(self.inner.unbounded_shr(3 - ith_bit as u32).bitand(1)),
            _ => None,
        }
    }

    /// Will return `None` if and only if `ith_bit` is greater than or equal to 4
    pub fn set_bit(&mut self, ith_bit: u8, bit: bit::Bit) -> Option<bit::Bit> {
        match ith_bit {
            ..4 => {
                let old =
                    bit::Bit::try_from_u8(self.inner.unbounded_shr(3 - ith_bit as u32).bitand(1));
                match bit {
                    bit::One => self.inner |= 1 << (3 - ith_bit),
                    bit::Zero => self.inner &= !(1 << (3 - ith_bit)),
                };
                old
            }
            _ => None,
        }
    }
}

impl std::fmt::Debug for Word {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match formatter.alternate() {
            false => write!(formatter, "{:1X}", self.inner),
            true => write!(formatter, "Word(0x{:1X})", self.inner),
        }
    }
}

pub fn pair_to_u8(high: Word, low: Word) -> u8 {
    use crate::storage::byte::Byte;
    Byte::new([high, low]).into_u8()
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn new() {
        use crate::bit::{One as One_, Zero};

        assert_eq!(Word::new([Zero, Zero, Zero, Zero]).inner, 0b0000);
        assert_eq!(Word::new([Zero, Zero, Zero, One_]).inner, 0b0001);
        assert_eq!(Word::new([Zero, Zero, One_, Zero]).inner, 0b0010);
        assert_eq!(Word::new([Zero, Zero, One_, One_]).inner, 0b0011);
        assert_eq!(Word::new([Zero, One_, Zero, Zero]).inner, 0b0100);
        assert_eq!(Word::new([Zero, One_, Zero, One_]).inner, 0b0101);
        assert_eq!(Word::new([Zero, One_, One_, Zero]).inner, 0b0110);
        assert_eq!(Word::new([Zero, One_, One_, One_]).inner, 0b0111);
        assert_eq!(Word::new([One_, Zero, Zero, Zero]).inner, 0b1000);
        assert_eq!(Word::new([One_, Zero, Zero, One_]).inner, 0b1001);
        assert_eq!(Word::new([One_, Zero, One_, Zero]).inner, 0b1010);
        assert_eq!(Word::new([One_, Zero, One_, One_]).inner, 0b1011);
        assert_eq!(Word::new([One_, One_, Zero, Zero]).inner, 0b1100);
        assert_eq!(Word::new([One_, One_, Zero, One_]).inner, 0b1101);
        assert_eq!(Word::new([One_, One_, One_, Zero]).inner, 0b1110);
        assert_eq!(Word::new([One_, One_, One_, One_]).inner, 0b1111);
    }

    #[test]
    fn into_bits() {
        use crate::bit::ITER_BIT;

        for a in ITER_BIT {
            for b in ITER_BIT {
                for c in ITER_BIT {
                    for d in ITER_BIT {
                        let bits = [a, b, c, d];
                        assert_eq!(Word::new(bits).into_bits(), bits)
                    }
                }
            }
        }
    }

    #[test]
    fn get_bit() {
        use bit::{Bit, One as One_, Zero};

        fn test(a: Bit, b: Bit, c: Bit, d: Bit) {
            let word = Word::new([a, b, c, d]);
            assert_eq!(word.get_bit(0), Some(a));
            assert_eq!(word.get_bit(1), Some(b));
            assert_eq!(word.get_bit(2), Some(c));
            assert_eq!(word.get_bit(3), Some(d));
            assert_eq!(word.get_bit(4), None);
        }

        test(Zero, Zero, Zero, One_);
        test(Zero, Zero, One_, Zero);
        test(Zero, One_, Zero, Zero);
        test(One_, Zero, Zero, Zero);
    }

    #[test]
    fn set_bit() {
        use bit::{Bit, One, Zero};

        fn test(fill: Bit, ith_bit: u8) {
            let word = Word::new([fill; 4]);
            let mut modify = word;
            let flip = fill.not();

            assert_eq!(
                modify.set_bit(ith_bit, fill),
                (ith_bit < 4).then_some(fill),
                "Setting bit to same bit"
            );
            assert_eq!(modify, word, "Checking equality after noop change");
            assert_eq!(
                modify.set_bit(ith_bit, flip),
                (ith_bit < 4).then_some(fill),
                "Setting bit to flipped bit"
            );
            match ith_bit {
                ..4 => assert_ne!(modify, word, "Checking inequality after change"),
                4.. => assert_eq!(modify, word, "Checking equality after failed change"),
            }
            assert_eq!(
                modify.get_bit(ith_bit),
                (ith_bit < 4).then_some(flip),
                "Checking flipped bit"
            );
        }

        test(One, 0);
        test(One, 1);
        test(One, 2);
        test(One, 3);
        test(One, 4);

        test(Zero, 0);
        test(Zero, 1);
        test(Zero, 2);
        test(Zero, 3);
        test(Zero, 4);
    }
}
