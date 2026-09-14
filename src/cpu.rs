use crate::storage::word::Word;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct CPU {
    stack_pointer: usize,
    flags: Flags,
    registers: [Word; CPU::REGISTER_COUNT],
}

impl CPU {
    pub const REGISTER_COUNT: usize = 8;

    pub fn new() -> CPU {
        CPU {
            stack_pointer: 0,
            flags: Flags {
                zero: false,
                sign: false,
                carry: false,
                overflow: false,
            },
            registers: [Word::ZEROED; CPU::REGISTER_COUNT],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flags {
    pub zero: bool,
    pub sign: bool,
    pub carry: bool,
    pub overflow: bool,
}
