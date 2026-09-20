use crate::bit;
use crate::manipulation as manip;
use crate::storage::{self, data::Address};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct CPU {
    instruction_pointer: Address,
    flags: Flags,
    registers: [storage::Word; CPU::REGISTER_COUNT],

    #[cfg(debug_assertions)]
    debug_analysys: bool,
}

impl CPU {
    pub const REGISTER_COUNT: usize = 15;

    pub fn new() -> CPU {
        CPU {
            instruction_pointer: Address::NULL,
            flags: Flags {
                zero: bit::Zero,
                sign: bit::Zero,
                carry: bit::Zero,
                overflow: bit::Zero,
                less: bit::Zero,
                equal: bit::Zero,
                greater: bit::Zero,
            },
            registers: [storage::Word::ZEROED; CPU::REGISTER_COUNT],

            #[cfg(debug_assertions)]
            debug_analysys: { todo!("Add debug analysys prompt") },
        }
    }

    pub fn run(&mut self, data: &mut storage::Data) -> Result<(), CpuError> {
        loop {
            let instruction = Instruction::fetch(self.instruction_pointer, data);

            use Instruction as I;
            let step: u8 = match instruction {
                I::Halt => break Ok(()),
                I::Move => self.handle_move(data)?,
                I::Compare => self.handle_compare(data)?,
                I::SignedCompare => self.handle_signed_compare(data)?,
                I::Jump => self.handle_jump(data)?,
                I::Skip => self.handle_skip(data)?,
                I::Get => self.handle_get(data)?,
                I::Set => self.handle_set(data)?,
                I::Add => self.handle_add(data)?,
                I::Sub => self.handle_sub(data)?,
                I::Mul => self.handle_mul(data)?,
                I::Div => self.handle_div(data)?,
                I::Not => self.handle_not(data)?,
                I::And => self.handle_and(data)?,
                I::Or => self.handle_or(data)?,
                I::Xor => self.handle_xor(data)?,
            };

            #[cfg(debug_assertions)]
            if self.debug_analysys {
                println!("{self:?}\n")
            }

            for _ in 0..step {
                if self.instruction_pointer.increment().0.is_one() {
                    return Err(CpuError::InstructionOverflow);
                }
            }
        }
    }

    fn handle_move(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        self.handle_biop(data, |in_register, out_register| {
            *in_register = out_register;
            None
        })
    }

    fn handle_compare(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        let lhs_register_id = data.get_word(self.instruction_pointer.offset(1)).into_u8() as usize;
        let rhs_register_id = data.get_word(self.instruction_pointer.offset(2)).into_u8() as usize;

        let Some(lhs_register) = self.registers.get(lhs_register_id) else {
            return Err(CpuError::InvalidRegister {
                address: self.instruction_pointer.offset(1),
                instruction: self.instruction_pointer,
            });
        };

        let (ret, compare) = match self.registers.get(rhs_register_id) {
            Some(rhs_register) => (2, rhs_register.into_uint()),
            None => (
                3,
                data.get_word(self.instruction_pointer.offset(3))
                    .into_uint(),
            ),
        };

        use std::cmp::Ordering as O;

        [self.flags.less, self.flags.equal, self.flags.greater] =
            match lhs_register.into_uint().cmp(compare) {
                O::Less => [bit::One, bit::Zero, bit::Zero],
                O::Equal => [bit::Zero, bit::One, bit::Zero],
                O::Greater => [bit::Zero, bit::Zero, bit::One],
            };

        Ok(ret + 1)
    }

    fn handle_signed_compare(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        todo!()
    }

    fn handle_jump(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        let high = data.get_word(self.instruction_pointer.offset(1));
        let low = data.get_word(self.instruction_pointer.offset(2));

        let jump_to = Address::new(high, low);
        self.instruction_pointer = jump_to;

        Ok(0)
    }

    fn handle_skip(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        todo!()
    }

    fn handle_get(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        let in_register_id = data.get_word(self.instruction_pointer.offset(1)).into_u8() as usize;
        let address = Address::new(
            data.get_word(self.instruction_pointer.offset(2)),
            data.get_word(self.instruction_pointer.offset(3)),
        );

        let Some(in_register) = self.registers.get_mut(in_register_id) else {
            return Err(CpuError::InvalidRegister {
                address: self.instruction_pointer.offset(1),
                instruction: self.instruction_pointer,
            });
        };

        *in_register = data.get_word(address);

        Ok(3)
    }

    fn handle_set(&mut self, data: &mut storage::Data) -> Result<u8, CpuError> {
        let in_register_id = data.get_word(self.instruction_pointer.offset(1)).into_u8() as usize;
        let address = Address::new(
            data.get_word(self.instruction_pointer.offset(2)),
            data.get_word(self.instruction_pointer.offset(3)),
        );

        let Some(in_register) = self.registers.get(in_register_id) else {
            return Err(CpuError::InvalidRegister {
                address: self.instruction_pointer.offset(1),
                instruction: self.instruction_pointer,
            });
        };

        data.set_word(address, *in_register);

        Ok(3)
    }

    fn handle_add(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        self.handle_biop(data, manip::modify_word(overflow_helper(manip::UInt::add)))
    }

    fn handle_sub(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        self.handle_biop(data, manip::modify_word(overflow_helper(manip::UInt::sub)))
    }

    fn handle_mul(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        self.handle_biop(data, manip::modify_word(overflow_helper(manip::UInt::mul)))
    }

    fn handle_div(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        todo!("implement `CPU::div`")
    }

    fn handle_not(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        self.handle_biop(
            data,
            manip::modify_word(no_overflow_helper(|_, val| val.not())),
        )
    }

    fn handle_and(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        self.handle_biop(
            data,
            manip::modify_word(no_overflow_helper(manip::UInt::and)),
        )
    }

    fn handle_or(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        self.handle_biop(
            data,
            manip::modify_word(no_overflow_helper(manip::UInt::or)),
        )
    }

    fn handle_xor(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        self.handle_biop(
            data,
            manip::modify_word(no_overflow_helper(manip::UInt::xor)),
        )
    }

    fn handle_biop(
        &mut self,
        data: &storage::Data,
        biop: impl Fn(&mut storage::Word, storage::Word) -> Option<manip::Overflow>,
    ) -> Result<u8, CpuError> {
        let in_register_id = data.get_word(self.instruction_pointer.offset(1)).into_u8() as usize;
        let out_register_id = data.get_word(self.instruction_pointer.offset(2)).into_u8() as usize;

        use std::slice::GetDisjointMutError as E;

        match self
            .registers
            .get_disjoint_mut([in_register_id, out_register_id])
        {
            Ok([in_register, out_register]) => {
                match biop(in_register, *out_register) {
                    Some(overflow) => self.flags.overflow = overflow.0,
                    None => (),
                }
                Ok(2 + 1)
            }
            Err(E::IndexOutOfBounds) if in_register_id != 0xF && out_register_id == 0xF => {
                self.registers[in_register_id] = data.get_word(self.instruction_pointer.offset(3));
                Ok(3 + 1)
            }
            Err(E::IndexOutOfBounds) => Err(CpuError::InvalidRegister {
                address: self.instruction_pointer.offset(1),
                instruction: self.instruction_pointer,
            }),
            Err(E::OverlappingIndices) => Ok(2 + 1),
        }
    }

    #[cfg(test)]
    fn create_and_run(slice: &[u8]) -> Result<CPU, CpuError> {
        let mut cpu = CPU::new();
        let mut data = storage::Data::from_slice(slice);
        cpu.run(&mut data).map(move |_| cpu)
    }
}

fn overflow_helper(
    f: impl Fn(manip::UInt, manip::UInt) -> (manip::UInt, manip::Overflow),
) -> impl Fn(manip::UInt, manip::UInt) -> (manip::UInt, Option<manip::Overflow>) {
    move |a, b| {
        let (output, overflow) = f(a, b);
        (output, Some(overflow))
    }
}

fn no_overflow_helper(
    f: impl Fn(manip::UInt, manip::UInt) -> manip::UInt,
) -> impl Fn(manip::UInt, manip::UInt) -> (manip::UInt, Option<manip::Overflow>) {
    move |a, b| (f(a, b), None)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flags {
    pub zero: bit::Bit,
    pub sign: bit::Bit,
    pub carry: bit::Bit,
    pub overflow: bit::Bit,
    pub less: bit::Bit,
    pub equal: bit::Bit,
    pub greater: bit::Bit,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum CpuError {
    InstructionOverflow,
    InvalidRegister {
        address: Address,
        instruction: Address,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    Halt = 0b0000,

    Move = 0b0001,

    Compare = 0b0010,
    SignedCompare = 0b0011,

    Jump = 0b0100,
    Skip = 0b0101,

    Get = 0b0110,
    Set = 0b0111,

    Add = 0b1000,
    Sub = 0b1001,
    Mul = 0b1010,
    Div = 0b1011,
    Not = 0b1100,
    And = 0b1101,
    Or = 0b1110,
    Xor = 0b1111,
}

impl Instruction {
    /// Gets the instruction at the selected position (No offset from true 0x0000)
    pub fn fetch(instruction_pointer: Address, data: &storage::Data) -> Instruction {
        use Instruction as I;
        use bit::{One as O, Zero as Z};

        match data.get_word(instruction_pointer).into_bits() {
            [Z, Z, Z, Z] => I::Halt,
            [Z, Z, Z, O] => I::Move,
            [Z, Z, O, Z] => I::Compare,
            [Z, Z, O, O] => I::SignedCompare,
            [Z, O, Z, Z] => I::Jump,
            [Z, O, Z, O] => I::Skip,
            [Z, O, O, Z] => I::Get,
            [Z, O, O, O] => I::Set,
            [O, Z, Z, Z] => I::Add,
            [O, Z, Z, O] => I::Sub,
            [O, Z, O, Z] => I::Mul,
            [O, Z, O, O] => I::Div,
            [O, O, Z, Z] => I::Not,
            [O, O, Z, O] => I::And,
            [O, O, O, Z] => I::Or,
            [O, O, O, O] => I::Xor,
        }
    }
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn halt() -> Result<(), CpuError> {
        CPU::create_and_run(&[]).map(std::mem::drop)
    }

    #[test]
    fn r#move() -> Result<(), CpuError> {
        //                                        r1 = 6      r2 = f;     r3 = r2;r4 = 3;
        let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x12, 0xff, 0x13, 0x21, 0x42])?;
        assert_eq!(final_state.registers[0x1].into_u8(), 0x6, "r1 == 6");
        assert_eq!(final_state.registers[0x2].into_u8(), 0xf, "r2 == f");
        assert_eq!(final_state.registers[0x3].into_u8(), 0xf, "r3 == f");
        assert_eq!(final_state.registers[0x4].into_u8(), 0x3, "r4 == 3");
        Ok(())
    }

    #[test]
    fn compare() -> Result<(), CpuError> {
        //                                        r1 = 0;     r2 = f;    cmp r1 r2
        let final_state = CPU::create_and_run(&[0x11, 0xf0, 0x12, 0xff, 0x21, 0x20])?;
        assert!(final_state.flags.less.is_one());
        assert!(final_state.flags.equal.is_zero());
        assert!(final_state.flags.greater.is_zero());

        //                                        r1 = 0;     r2 = f;    cmp r2 r1
        let final_state = CPU::create_and_run(&[0x11, 0xf0, 0x12, 0xff, 0x22, 0x10])?;
        assert!(final_state.flags.less.is_zero());
        assert!(final_state.flags.equal.is_zero());
        assert!(final_state.flags.greater.is_one());

        //                                        r1 = 6;     r2 = 6;    cmp r1 r2
        let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x12, 0xf6, 0x21, 0x20])?;
        assert!(final_state.flags.less.is_zero());
        assert!(final_state.flags.equal.is_one());
        assert!(final_state.flags.greater.is_zero());

        //                                        r1 = 6;    cmp r1 7
        let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x21, 0xf7])?;
        assert!(final_state.flags.less.is_one());
        assert!(final_state.flags.equal.is_zero());
        assert!(final_state.flags.greater.is_zero());

        //                                        r1 = 6;    cmp r1 6
        let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x21, 0xf6])?;
        assert!(final_state.flags.less.is_zero());
        assert!(final_state.flags.equal.is_one());
        assert!(final_state.flags.greater.is_zero());

        //                                        r1 = 6;    cmp r1 5
        let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x21, 0xf5])?;
        assert!(final_state.flags.less.is_zero());
        assert!(final_state.flags.equal.is_zero());
        assert!(final_state.flags.greater.is_one());

        Ok(())
    }

    #[test]
    fn set() -> Result<(), CpuError> {
        let mut cpu = CPU::new();
        let mut data = storage::Data::from_slice(&[0x11, 0xff, 0x61, 0x66]);
        cpu.run(&mut data)?;
        assert_eq!(data.get_word(Address::from_u8(0x66)).into_u8(), 0xf);
        Ok(())
    }

    #[test]
    fn get() -> Result<(), CpuError> {
        let _final_state = CPU::create_and_run(&[0x6])?;
        todo!("implement get test")
    }
}
