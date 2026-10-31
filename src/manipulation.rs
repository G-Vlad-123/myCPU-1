use tap::Pipe;

use crate::bit;
use crate::storage;

pub fn modify_word(
    uint_op: impl Fn(UInt, UInt) -> (UInt, Option<Overflow>),
) -> impl Fn(&mut storage::Word, storage::Word) -> Option<Overflow> {
    move |register, value| {
        let (new_register_value, overflow) = uint_op(register.into_uint(), value.into_uint());
        *register = new_register_value.into_word();
        overflow
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UInt {
    inner: [bit::Bit; 4],
}

impl UInt {
    pub const ZERO: UInt = UInt {
        inner: [bit::Zero; 4],
    };

    #[must_use]
    pub fn from_bits(bits: [bit::Bit; 4]) -> UInt {
        UInt { inner: bits }
    }

    #[must_use]
    pub fn from_word(word: storage::Word) -> UInt {
        UInt::from_bits(word.into_bits())
    }

    #[must_use]
    #[cfg(test)]
    pub fn from_u8(u8: u8) -> UInt {
        UInt::from_word(storage::Word::from_u8(u8))
    }

    #[must_use]
    pub fn into_bits(self) -> [bit::Bit; 4] {
        self.inner
    }

    #[must_use]
    pub fn into_word(self) -> storage::Word {
        storage::Word::new(self.into_bits())
    }

    #[must_use]
    pub fn increment(&mut self) -> Overflow {
        let mut overflow: bit::Bit = bit::One;

        for idx in (0usize..4).rev() {
            [self.inner[idx], overflow] =
                [self.inner[idx].xor(overflow), self.inner[idx].and(overflow)]
        }

        Overflow(overflow)
    }

    #[must_use]
    #[cfg(test)]
    pub fn decrement(&mut self) -> Overflow {
        let mut underflow: bit::Bit = bit::One;

        for idx in (0usize..4).rev() {
            [self.inner[idx], underflow] = [
                self.inner[idx].xor(underflow),
                self.inner[idx].not().and(underflow),
            ]
        }

        Overflow(underflow)
    }

    #[must_use]
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

    #[must_use]
    /// signed compare
    pub fn smp(mut self, mut rhs: UInt) -> std::cmp::Ordering {
        self.inner[0] = self.inner[0].not();
        rhs.inner[0] = rhs.inner[0].not();

        self.cmp(rhs)
    }

    #[must_use]
    fn add_internal(self, rhs: UInt, overflow: bit::Bit) -> (UInt, Overflow) {
        let (d, o) = bit_add_with_overflow(self.inner[3], rhs.inner[3], Overflow(overflow));
        let (c, o) = bit_add_with_overflow(self.inner[2], rhs.inner[2], o);
        let (b, o) = bit_add_with_overflow(self.inner[1], rhs.inner[1], o);
        let (a, o) = bit_add_with_overflow(self.inner[0], rhs.inner[0], o);
        (
            UInt {
                inner: [a, b, c, d],
            },
            o,
        )
    }

    #[must_use]
    pub fn add(self, rhs: UInt) -> (UInt, Overflow) {
        self.add_internal(rhs, bit::Zero)
    }

    #[must_use]
    pub fn sub(self, rhs: UInt) -> (UInt, Overflow) {
        self.add_internal(rhs.not(), bit::One)
            .pipe(|(uint, overflow)| (uint, Overflow(overflow.0.not())))
    }

    #[must_use]
    pub fn mul(self, rhs: UInt) -> (UInt, Overflow) {
        let mut ret = if rhs.inner[3].is_one() {
            self
        } else {
            UInt::ZERO
        };

        let mut overflow = Overflow(bit::Zero);

        let mut add = |idx: usize| {
            if rhs.inner[idx].is_one() {
                let (shl, mut ret_overflow) = self.shl_impl(3 - idx);
                if ret_overflow.0.is_one() {
                    overflow = Overflow(bit::One);
                }

                (ret, ret_overflow) = ret.add(shl);
                if ret_overflow.0.is_one() {
                    overflow = Overflow(bit::One);
                }
            }
        };

        add(2);
        add(1);
        add(0);

        (ret, overflow)
    }

    #[must_use]
    /// # PANIC
    /// If `rhs` represents `0`
    pub fn div_rem(self, rhs: UInt) -> (UInt, UInt) {
        assert!(rhs != UInt::ZERO, "Divided {self:?} by 0");

        let mut reminder = self;
        let mut result = UInt::ZERO;

        let mut sub = |idx: usize| match reminder.shr_impl(idx).0.cmp(rhs) {
            std::cmp::Ordering::Less => (),
            std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => {
                reminder = reminder.sub(rhs.shl_impl(idx).0).0;
                result.inner[3 - idx] = bit::One;
            }
        };

        for idx in (0..rhs.bit_length()).rev() {
            sub(idx)
        }

        (result, reminder)
    }

    #[must_use]
    pub fn not(self) -> UInt {
        UInt {
            inner: self.inner.map(bit::Bit::not),
        }
    }

    #[must_use]
    pub fn and(self, rhs: UInt) -> UInt {
        let op = |idx| bit::Bit::and(self.inner[idx], rhs.inner[idx]);

        UInt {
            inner: std::array::from_fn(op),
        }
    }

    #[must_use]
    pub fn or(self, rhs: UInt) -> UInt {
        let op = |idx| bit::Bit::or(self.inner[idx], rhs.inner[idx]);

        UInt {
            inner: std::array::from_fn(op),
        }
    }

    #[must_use]
    pub fn xor(self, rhs: UInt) -> UInt {
        let op = |idx| bit::Bit::xor(self.inner[idx], rhs.inner[idx]);

        UInt {
            inner: std::array::from_fn(op),
        }
    }

    #[must_use]
    #[allow(unused)]
    fn shr_impl(self, rhs: usize) -> (UInt, Overflow) {
        let mut overflow = Overflow(bit::Zero);
        let mut new = [bit::Zero; 4];

        for (idx, bit) in self.inner.into_iter().enumerate().rev() {
            match new.get_mut(idx.saturating_add(rhs as usize)) {
                Some(new_bit) => *new_bit = bit,
                None if bit.is_one() => overflow = Overflow(bit::One),
                None => (),
            }
        }

        (UInt { inner: new }, overflow)
    }

    #[must_use]
    fn shl_impl(self, rhs: usize) -> (UInt, Overflow) {
        let mut overflow = Overflow(bit::Zero);
        let mut new = [bit::Zero; 4];

        for (idx, bit) in self.inner.into_iter().enumerate().rev() {
            match new.get_mut(idx.wrapping_sub(rhs as usize)) {
                Some(new_bit) => *new_bit = bit,
                None if bit.is_one() => overflow = Overflow(bit::One),
                None => (),
            }
        }

        (UInt { inner: new }, overflow)
    }

    #[must_use]
    /// Returns how many bits the number takes up.
    fn bit_length(self) -> usize {
        for idx in (0..4).rev() {
            if self.inner[idx].is_one() {
                return idx + 1;
            }
        }

        0
    }
}

#[must_use]
fn bit_add_with_overflow(
    lhs: bit::Bit,
    rhs: bit::Bit,
    Overflow(overflow): Overflow,
) -> (bit::Bit, Overflow) {
    let (a, b, c) = (lhs.is_one(), rhs.is_one(), overflow.is_one());
    let bit = bit::Bit::from_bool;
    (
        bit(a ^ b ^ c),
        Overflow(bit((a && b) || (a && c) || (b && c))),
    )
}

impl std::fmt::Debug for UInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match f.alternate() {
            false => write!(f, "UInt({})", self.into_word()),
            true => write!(f, "UInt{:?}", self.inner),
        }
    }
}

impl std::fmt::Display for UInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.into_word().fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overflow(pub bit::Bit);

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

    #[test]
    fn add() {
        fn test(x: u8, y: u8) {
            let a = UInt::from_u8(x);
            let b = UInt::from_u8(y);
            let (c, o) = a.add(b);
            assert_eq!(c, UInt::from_u8((x + y) % 16));
            assert_eq!(o.0.is_zero(), x + y < 16)
        }

        for x in 0..16 {
            for y in 0..16 {
                test(x, y)
            }
        }
    }

    #[test]
    fn sub() {
        fn test(x: u8, y: u8) {
            let a = UInt::from_u8(x);
            let b = UInt::from_u8(y);
            let (c, o) = a.sub(b);
            assert_eq!(b.add(c).0, a, "failed to subtract x = {x} and y = {y}");

            use std::cmp::Ordering;
            match a.cmp(b) {
                Ordering::Less => assert_eq!(o.0, bit::One, "overflow = false for {x} - {y}"),
                Ordering::Greater | Ordering::Equal => {
                    assert_eq!(o.0, bit::Zero, "overflow = true for {x} - {y}")
                }
            }
        }

        for x in 0..16 {
            for y in 0..16 {
                test(x, y)
            }
        }
    }

    #[test]
    fn mul() {
        fn test(x: u8, y: u8) {
            let a = UInt::from_u8(x);
            let b = UInt::from_u8(y);
            let (c, o) = a.mul(b);
            let (z, o2) = ((x * y) % 16, Overflow(bit::Bit::from_bool(x * y >= 16)));

            assert_eq!(
                c,
                UInt::from_u8(z),
                "failed to multiply x = {x} and y = {y}"
            );
            assert_eq!(o, o2, "overflow = {} for {x} * {y}", o2.0.is_one());
        }

        for x in 0..16 {
            for y in 0..16 {
                test(x, y)
            }
        }
    }

    #[test]
    fn div_rem_non_zero() {
        fn test(x: u8, y: u8) {
            let a = UInt::from_u8(x);
            let b = UInt::from_u8(y);
            let (result, reminder) = a.div_rem(b);

            assert_eq!(result, UInt::from_u8(x / y), "{x} / {y}");
            assert_eq!(reminder, UInt::from_u8(x % y), "{x} % {y}");
        }

        for x in 0..16 {
            for y in 1..16 {
                test(x, y)
            }
        }
    }

    #[test]
    #[should_panic = "Divided 0 by 0"]
    fn div_rem_zero() {
        let _ = UInt::ZERO.div_rem(UInt::ZERO);
    }
}
