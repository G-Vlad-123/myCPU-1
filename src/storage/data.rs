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
    fn compute(idx: usize) -> Idx {
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
    pub fn get_word(&self, idx: usize) -> Word {
        let idx = Idx::compute(idx);

        self.inner[idx.byte]
            .get_word(idx.word)
            .expect("`idx.word` is guaranteed to be 0 or 1")
    }

    pub fn set_word(&mut self, idx: usize, new: Word) -> Word {
        let idx = Idx::compute(idx);

        self.inner[idx.byte]
            .set_word(new, idx.word)
            .expect("`idx.word` is guaranteed to be 0 or 1")
    }
}
