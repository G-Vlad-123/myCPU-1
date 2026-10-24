use crate::bit;
use crate::manipulation as manip;
use crate::storage::{self, data::Address};

#[cfg(all(feature = "force-debug", feature = "disable-debug"))]
compile_error!("Can't both forcefully disable and enable debug analysys!");

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct CPU {
    instruction_pointer: Address,
    flags: Flags,
    registers: [storage::Word; CPU::REGISTER_COUNT],

    #[cfg(debug_assertions)]
    debug_analysys: bool,
}

fn input(prompt: &str, expected: &[&str]) -> String {
    println!("{prompt}");
    let mut read = String::new();
    loop {
        match std::io::stdin().read_line(&mut read) {
            Ok(_) => match expected.contains(&read.trim()) {
                true => {
                    read.truncate(read.trim().len());
                    return read;
                }
                false => println!(
                    "Invalid syntax, only items from the following list are allowed:\n{expected:?}"
                ),
            },
            Err(err) => println!("\nI/O error: {err}\n"),
        }
    }
}

const INSTRUCTION_SIZE: u8 = 1;

impl CPU {
    pub const REGISTER_COUNT: usize = 15;

    pub fn new() -> CPU {
        CPU {
            instruction_pointer: Address::NULL,
            flags: Flags::ZEROED,
            registers: [storage::Word::ZEROED; CPU::REGISTER_COUNT],

            #[cfg(debug_assertions)]
            debug_analysys: if cfg!(any(feature = "force-debug", feature = "disable-debug")) {
                cfg!(feature = "force-debug")
            } else {
                match input("Enable debug analysys (Y/n):", &["", "y", "Y", "n", "N"]).as_str() {
                    "" | "y" | "Y" => true,
                    "n" | "N" => false,
                    _ => unreachable!(),
                }
            },
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
            'dbg: {
                if !self.debug_analysys {
                    break 'dbg;
                }

                let cont_response = input("Continue (Y/n):", &["", "y", "Y", "n", "N"]);

                println!("\n{self:#?}\n");

                match cont_response.as_str() {
                    "n" | "N" => (),
                    _ => break 'dbg,
                }

                return Ok(());
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

        [self.flags.equal, self.flags.greater] = match lhs_register.into_uint().cmp(compare) {
            O::Less => [bit::Zero, bit::Zero],
            O::Equal => [bit::One, bit::Zero],
            O::Greater => [bit::Zero, bit::One],
        };

        Ok(ret + INSTRUCTION_SIZE)
    }

    fn handle_signed_compare(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
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

        [self.flags.equal, self.flags.greater] = match lhs_register.into_uint().smp(compare) {
            O::Less => [bit::Zero, bit::Zero],
            O::Equal => [bit::One, bit::Zero],
            O::Greater => [bit::Zero, bit::One],
        };

        Ok(ret + INSTRUCTION_SIZE)
    }

    fn handle_jump(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        let high = data.get_word(self.instruction_pointer.offset(1));
        let low = data.get_word(self.instruction_pointer.offset(2));

        self.instruction_pointer.jump_to(high, low);

        Ok(0)
    }

    fn handle_skip(&mut self, data: &storage::Data) -> Result<u8, CpuError> {
        let [zero_division, overflow, equal, greater] = data
            .get_word(self.instruction_pointer.offset(1))
            .into_bits();

        if bit::Zero
            .or(self.flags.zero_divison.and(zero_division))
            .or(self.flags.overflow.and(overflow))
            .or(self.flags.equal.and(equal))
            .or(self.flags.greater.and(greater))
            .is_zero()
        {
            return Ok(1 + INSTRUCTION_SIZE);
        }

        let biop = || {
            let out_register_id = data
                .get_word(self.instruction_pointer.offset(INSTRUCTION_SIZE + 1 + 2))
                .into_u8() as usize;

            let out_register = self.registers.get(out_register_id);

            match out_register {
                Some(_) => 2,
                None => 3,
            }
        };

        use Instruction as I;
        let input_count: u8 = match I::fetch(self.instruction_pointer.offset(2), data) {
            I::Halt => 0,
            I::Move => biop(),
            I::Compare => 2,
            I::SignedCompare => 2,
            I::Jump => 2,
            I::Skip => 1,
            I::Get => 3,
            I::Set => 3,
            I::Add => biop(),
            I::Sub => biop(),
            I::Mul => biop(),
            I::Div => {
                let out_register_id = data
                    .get_word(self.instruction_pointer.offset(INSTRUCTION_SIZE + 1 + 3))
                    .into_u8() as usize;

                let out_register = self.registers.get(out_register_id);

                match out_register {
                    Some(_) => 3,
                    None => 4,
                }
            }
            I::Not => biop(),
            I::And => biop(),
            I::Or => biop(),
            I::Xor => biop(),
        };

        Ok(input_count + 1 + 2 * INSTRUCTION_SIZE)
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

        Ok(3 + INSTRUCTION_SIZE)
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

        Ok(3 + INSTRUCTION_SIZE)
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
        let div_register_id = data.get_word(self.instruction_pointer.offset(1)).into_u8() as usize;
        let rem_register_id = data.get_word(self.instruction_pointer.offset(2)).into_u8() as usize;
        let out_register_id = data.get_word(self.instruction_pointer.offset(3)).into_u8() as usize;

        self.flags.zero_divison = bit::Zero;

        fn division(
            div: &mut storage::Word,
            rem: Option<&mut storage::Word>,
            by: storage::Word,
            overflow: &mut bit::Bit,
        ) {
            let result = div.into_uint().div_rem(by.into_uint());
            *div = result.0.into_word();
            if let Some(rem) = rem {
                *rem = result.1.into_word();
            };
            *overflow = result.2.0;
        }

        let invalid_register = |offset| CpuError::InvalidRegister {
            address: self.instruction_pointer.offset(offset),
            instruction: self.instruction_pointer,
        };

        let invalid_register_disjoint = move |_: std::slice::GetDisjointMutError| {
            invalid_register(if div_register_id == 0xF { 1 } else { 2 })
        };

        if div_register_id == rem_register_id && div_register_id == out_register_id {
            let divisor = self
                .registers
                .get_mut(div_register_id)
                .ok_or(invalid_register(1))?;
            division(divisor, None, *divisor, &mut self.flags.overflow);
            return Ok(3 + INSTRUCTION_SIZE);
        }

        if div_register_id == rem_register_id {
            let (dividant, arg_count) = match self.registers.get(out_register_id) {
                Some(&dividant) => (dividant, 3),
                None => (data.get_word(self.instruction_pointer.offset(4)), 4),
            };

            let divisor = self
                .registers
                .get_mut(div_register_id)
                .ok_or(invalid_register(1))?;

            division(divisor, None, dividant, &mut self.flags.overflow);

            return Ok(arg_count + INSTRUCTION_SIZE);
        }

        if div_register_id == out_register_id {
            let [divisor, reminder] = self
                .registers
                .get_disjoint_mut([div_register_id, rem_register_id])
                .map_err(invalid_register_disjoint)?;

            division(divisor, Some(reminder), *divisor, &mut self.flags.overflow);

            return Ok(3 + INSTRUCTION_SIZE);
        }

        if rem_register_id == out_register_id {
            let [divisor, reminder] = self
                .registers
                .get_disjoint_mut([div_register_id, rem_register_id])
                .map_err(invalid_register_disjoint)?;

            let divident = *reminder;
            division(divisor, Some(reminder), divident, &mut self.flags.overflow);

            return Ok(3 + INSTRUCTION_SIZE);
        }

        if out_register_id == 0xF {
            let [divisor, reminder] = self
                .registers
                .get_disjoint_mut([div_register_id, rem_register_id])
                .map_err(invalid_register_disjoint)?;

            let divident = data.get_word(self.instruction_pointer.offset(4));
            division(divisor, Some(reminder), divident, &mut self.flags.overflow);

            return Ok(4 + INSTRUCTION_SIZE);
        }

        let [divisor, reminder, &mut divident] = self
            .registers
            .get_disjoint_mut([div_register_id, rem_register_id, out_register_id])
            .map_err(invalid_register_disjoint)?;

        division(divisor, Some(reminder), divident, &mut self.flags.overflow);

        Ok(3 + INSTRUCTION_SIZE)
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

        let mut biop = |in_register, out_register| match biop(in_register, out_register) {
            Some(overflow) => self.flags.overflow = overflow.0,
            None => (),
        };

        if in_register_id == out_register_id {
            return match self.registers.get_mut(in_register_id) {
                Some(register) => {
                    biop(register, *register);
                    Ok(2 + INSTRUCTION_SIZE)
                }
                None => Err(CpuError::InvalidRegister {
                    address: self.instruction_pointer.offset(1),
                    instruction: self.instruction_pointer,
                }),
            };
        }

        if out_register_id == 0xF {
            return match self.registers.get_mut(in_register_id) {
                Some(register) => {
                    let value = data.get_word(self.instruction_pointer.offset(3));
                    biop(register, value);
                    Ok(3 + INSTRUCTION_SIZE)
                }
                None => Err(CpuError::InvalidRegister {
                    address: self.instruction_pointer.offset(1),
                    instruction: self.instruction_pointer,
                }),
            };
        }

        match self
            .registers
            .get_disjoint_mut([in_register_id, out_register_id])
        {
            Ok([in_register, &mut out_register]) => {
                biop(in_register, out_register);
                Ok(2 + INSTRUCTION_SIZE)
            }
            Err(_) => Err(CpuError::InvalidRegister {
                address: self.instruction_pointer.offset(1),
                instruction: self.instruction_pointer,
            }),
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
    pub greater: bit::Bit,
    pub equal: bit::Bit,
    pub overflow: bit::Bit,
    pub zero_divison: bit::Bit,
}

impl Flags {
    pub const ZEROED: Flags = Flags {
        greater: bit::Zero,
        equal: bit::Zero,
        overflow: bit::Zero,
        zero_divison: bit::Zero,
    };
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
        let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x12, 0xff, 0x13, 0x21, 0x4f, 0x30])?;
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
        assert!(final_state.flags.equal.is_zero());
        assert!(final_state.flags.greater.is_zero());

        //                                        r1 = 0;     r2 = f;    cmp r2 r1
        let final_state = CPU::create_and_run(&[0x11, 0xf0, 0x12, 0xff, 0x22, 0x10])?;
        assert!(final_state.flags.equal.is_zero());
        assert!(final_state.flags.greater.is_one());

        //                                        r1 = 6;     r2 = 6;    cmp r1 r2
        let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x12, 0xf6, 0x21, 0x20])?;
        assert!(final_state.flags.equal.is_one());
        assert!(final_state.flags.greater.is_zero());

        //                                        r1 = 6;    cmp r1 7
        let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x21, 0xf7])?;
        assert!(final_state.flags.equal.is_zero());
        assert!(final_state.flags.greater.is_zero());

        //                                        r1 = 6;    cmp r1 6
        let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x21, 0xf6])?;
        assert!(final_state.flags.equal.is_one());
        assert!(final_state.flags.greater.is_zero());

        //                                        r1 = 6;    cmp r1 5
        let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x21, 0xf5])?;
        assert!(final_state.flags.equal.is_zero());
        assert!(final_state.flags.greater.is_one());

        Ok(())
    }

    #[test]
    #[allow(unreachable_code)]
    fn signed_compare() -> Result<(), CpuError> {
        // NOTE: 8..=f are negative numbers, they represent the -8..=-1 (in the same order)
        //       This also means that MAX = 7 and MIN = 8

        //                                        r1 = 0;     r2 = f;    smp r2 r1
        let final_state = CPU::create_and_run(&[0x11, 0xf0, 0x12, 0xff, 0x32, 0x10])?;
        assert!(final_state.flags.equal.is_zero(), "smp f 0 -> not equal");
        assert!(
            final_state.flags.greater.is_zero(),
            "smp f 0 -> not greater"
        );

        //                                        r1 = 0;     r2 = f;    smp r1 r2
        let final_state = CPU::create_and_run(&[0x11, 0xf0, 0x12, 0xff, 0x31, 0x20])?;
        assert!(final_state.flags.equal.is_zero(), "smp 0 f -> not equal");
        assert!(final_state.flags.greater.is_one(), "smp 0 f -> greater");

        //                                        r1 = 6;     r2 = 6;    smp r1 r2
        let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x12, 0xf6, 0x31, 0x20])?;
        assert!(final_state.flags.equal.is_one(), "smp 6 6 -> equal");
        assert!(
            final_state.flags.greater.is_zero(),
            "smp 6 6 -> not greater"
        );

        //                                        r1 = 7;    smp r1 8
        let final_state = CPU::create_and_run(&[0x11, 0xf7, 0x31, 0xf8])?;
        assert!(final_state.flags.equal.is_zero(), "smp 7 8 -> not equal");
        assert!(final_state.flags.greater.is_one(), "smp 7 8 -> greater");

        //                                        r1 = 7;    smp r1 7
        let final_state = CPU::create_and_run(&[0x11, 0xf7, 0x31, 0xf7])?;
        assert!(final_state.flags.equal.is_one(), "smp 6 6 -> equal");
        assert!(
            final_state.flags.greater.is_zero(),
            "smp 6 6 -> not greater"
        );

        //                                        r1 = 7;    smp r1 6
        let final_state = CPU::create_and_run(&[0x11, 0xf7, 0x31, 0xf6])?;
        assert!(final_state.flags.equal.is_zero(), "smp 6 5 -> not equal");
        assert!(final_state.flags.greater.is_one(), "smp 6 5 -> greater");

        //                                        r1 = 5;    smp r1 6
        let final_state = CPU::create_and_run(&[0x11, 0xf5, 0x31, 0xf6])?;
        assert!(final_state.flags.equal.is_zero(), "smp 5 6 -> not equal");
        assert!(
            final_state.flags.greater.is_zero(),
            "smp 5 6 -> not greater"
        );

        Ok(())
    }

    #[test]
    fn set() -> Result<(), CpuError> {
        let mut cpu = CPU::new();
        let mut data = storage::Data::from_slice(&[0x11, 0xff, 0x71, 0x66]);
        cpu.run(&mut data)?;
        assert_eq!(data.get_word(Address::from_u8(0x66)).into_u8(), 0xf);
        Ok(())
    }

    #[test]
    fn get() -> Result<(), CpuError> {
        let mut cpu = CPU::new();
        let mut data = storage::Data::from_slice(&[0x11, 0xff, 0x71, 0x66, 0x62, 0x66]);
        cpu.run(&mut data)?;
        assert_eq!(cpu.registers[2].into_u8(), 0xf);
        Ok(())
    }

    #[test]
    #[allow(unused_labels)]
    fn biop() -> Result<(), CpuError> {
        let r1 = 6;
        let r2 = 2;

        'addition: {
            //                                        r1 = 6;     r2 = 2;    r1 += r2;
            let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x12, 0xf2, 0x81, 0x20])?;
            assert_eq!(final_state.registers[1].into_u8(), r1 + r2);
            assert_eq!(final_state.flags.overflow, bit::Zero);

            //                                        r1 = 6;    r1 += r1;
            let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x81, 0x10])?;
            assert_eq!(final_state.registers[1].into_u8(), r1 + r1);
            assert_eq!(final_state.flags.overflow, bit::Zero);

            //                                        r1 = 6;    r1 += 2;
            let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x81, 0xf2])?;
            assert_eq!(final_state.registers[1].into_u8(), r1 + 2);
            assert_eq!(final_state.flags.overflow, bit::Zero);

            //                                        r1 = f;     r2 = 1;    r1 += r2;
            let final_state = CPU::create_and_run(&[0x11, 0xff, 0x12, 0xf1, 0x81, 0x20])?;
            assert_eq!(final_state.registers[1].into_u8(), 0);
            assert_eq!(final_state.flags.overflow, bit::One);

            //                                        r1 = f;    r1 += r1;
            let final_state = CPU::create_and_run(&[0x11, 0xff, 0x81, 0x10])?;
            assert_eq!(final_state.registers[1].into_u8(), 0xe);
            assert_eq!(final_state.flags.overflow, bit::One);

            //                                        r1 = f;    r1 += 1;
            let final_state = CPU::create_and_run(&[0x11, 0xff, 0x81, 0xf1])?;
            assert_eq!(final_state.registers[1].into_u8(), 0);
            assert_eq!(final_state.flags.overflow, bit::One);
        }

        Ok(())
    }

    #[test]
    #[rustfmt::skip]
    fn division() -> Result<(), CpuError> {
        // d(a, b, c) = `a = a / c and b = a % c`

        //                                        r1 = 7;     r3 = 2;   d(r1, r2, r3);
        let final_state = CPU::create_and_run(&[0x11, 0xf7, 0x13, 0xf2, 0xb1, 0x23])?;
        assert_eq!(final_state.registers[1].into_u8(), 7 / 2, "7 / 2");
        assert_eq!(final_state.registers[2].into_u8(), 7 % 2, "7 % 2");

        //                                        r1 = 6;     r3 = 2;   d(r1, r2, r3);
        let final_state = CPU::create_and_run(&[0x11, 0xf6, 0x13, 0xf2, 0xb1, 0x23])?;
        assert_eq!(final_state.registers[1].into_u8(), 6 / 2, "6 / 2");
        assert_eq!(final_state.registers[2].into_u8(), 6 % 2, "6 % 2");

        //                                        r1 = 3;     r3 = 5;   d(r1, r2, r3);
        let final_state = CPU::create_and_run(&[0x11, 0xf3, 0x13, 0xf5, 0xb1, 0x23])?;
        assert_eq!(final_state.registers[1].into_u8(), 3 / 5, "3 / 5");
        assert_eq!(final_state.registers[2].into_u8(), 3 % 5, "3 % 5");

        //                                        r1 = 4;     r3 = 4;   d(r1, r2, r3);
        let final_state = CPU::create_and_run(&[0x11, 0xf4, 0x13, 0xf4, 0xb1, 0x23])?;
        assert_eq!(final_state.registers[1].into_u8(), 4 / 4, "4 / 4");
        assert_eq!(final_state.registers[2].into_u8(), 4 % 4, "4 % 4");

        //                                        r1 = 7;   d(r1, r2, 2);
        let final_state = CPU::create_and_run(&[0x11, 0xf7, 0xb1, 0x2f, 0x20])?;
        assert_eq!(final_state.registers[1].into_u8(), 7 / 2, "/: const divident");
        assert_eq!(final_state.registers[2].into_u8(), 7 % 2, "%: const divident");

        //                                        r1 = 7;     r3 = 2;   d(r1, r1, r3);
        let final_state = CPU::create_and_run(&[0x11, 0xf7, 0x13, 0xf2, 0xb1, 0x23])?;
        assert_eq!(final_state.registers[1].into_u8(), 7 / 2, "/: div == rem");
        assert_eq!(final_state.registers[2].into_u8(), 7 % 2, "%: div == rem");

        //                                        r1 = 7;   d(r1, r1, 2);
        let final_state = CPU::create_and_run(&[0x11, 0xf7, 0xb1, 0x1f, 0x20])?;
        assert_eq!(final_state.registers[1].into_u8(), 7 / 7, "/: div == rem && const divident");
        assert_eq!(final_state.registers[2].into_u8(), 7 % 7, "%: div == rem && const divident");

        //                                        r1 = 7;     r3 = 0;   d(r1, r2, r3);
        let final_state = CPU::create_and_run(&[0x11, 0xf7, 0x13, 0xf0, 0xb1, 0x23])?;
        assert_eq!(final_state.registers[1].into_u8(), 7, "/: divident == 0");
        assert_eq!(final_state.registers[2].into_u8(), 0, "%: divident == 0");
        assert_eq!(final_state.flags.zero_divison, bit::One, "zero division flag");

        //                                        r1 = 7;   d(r1, r2, 0);
        let final_state = CPU::create_and_run(&[0x11, 0xf7, 0xb1, 0x2f, 0x00])?;
        assert_eq!(final_state.registers[1].into_u8(), 7, "/: const divident == 0");
        assert_eq!(final_state.registers[2].into_u8(), 0, "%: const divident == 0");
        assert_eq!(final_state.flags.zero_divison, bit::One, "const zero division flag");

        //                                        r1 = 7;     r3 = 0;   d(r1, r1, r3);
        let final_state = CPU::create_and_run(&[0x11, 0xf7, 0x13, 0xf0, 0xb1, 0x13])?;
        assert_eq!(final_state.registers[1].into_u8(), 7, "/: div == rem && divident == 0");
        assert_eq!(final_state.registers[2].into_u8(), 0, "%: div == rem && divident == 0");
        assert_eq!(final_state.flags.zero_divison, bit::One, "div == rem && zero division flag");

        //                                        r1 = 7;   d(r1, r1, 0);
        let final_state = CPU::create_and_run(&[0x11, 0xf7, 0xb1, 0x1f, 0x00])?;
        assert_eq!(final_state.registers[1].into_u8(), 7, "/: div == rem && const divident == 0");
        assert_eq!(final_state.registers[2].into_u8(), 0, "%: div == rem && const divident == 0");
        assert_eq!(final_state.flags.zero_divison, bit::One, "div == rem && const zero division flag");

        Ok(())
    }

    #[test]
    #[allow(unused_labels)]
    fn skip() -> Result<(), CpuError> {
        let _final_state = CPU::create_and_run(&[])?;
        todo!("test cpu::testing::skip")
    }
}
