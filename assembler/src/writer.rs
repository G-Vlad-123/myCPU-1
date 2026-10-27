use crate::tokens::{
    Instruction as Instr,
    values::{AppendTo, Word},
};

#[derive(Debug, snafu::Snafu)]
pub enum WritingError {
    #[snafu(context(false))]
    UnableToWriteToTarget {
        #[snafu(source(from(exact)))]
        source: std::io::Error,
    },

    UnableToWriteEverything,
}

pub fn write(
    target: impl std::io::Write,
    program: impl Iterator<Item = Instr>,
) -> Result<(), WritingError> {
    write_data(target, collect_data(program))
}

fn collect_data(program: impl Iterator<Item = Instr>) -> Vec<Word> {
    let mut data: Vec<Word> = Vec::new();

    macro_rules! append_to_data {
        ( $($word:expr),+ ) => {
            {$(
                $word.append_to(&mut data);
            )+}
        };
    }

    for instruction in program {
        match instruction {
            Instr::Halt => append_to_data!(Word::W0),
            Instr::Move { register, value } => append_to_data!(Word::W1, register, value),
            Instr::Compare { register, value } => append_to_data!(Word::W2, register, value),
            Instr::SignedCompare { register, value } => append_to_data!(Word::W3, register, value),
            Instr::Jump { address } => append_to_data!(Word::W4, address),
            Instr::Skip {
                due_to_greater: a,
                due_to_equal: b,
                due_to_overflow: c,
                due_to_zero_division: d,
            } => append_to_data!(Word::W5, [a, b, c, d]),
            Instr::Get { register, address } => {
                append_to_data!(Word::W6, register, address)
            }
            Instr::Set { register, address } => {
                append_to_data!(Word::W7, register, address)
            }
            Instr::Add { register, value } => append_to_data!(Word::W8, register, value),
            Instr::Sub { register, value } => append_to_data!(Word::W9, register, value),
            Instr::Mul { register, value } => append_to_data!(Word::WA, register, value),
            Instr::Div {
                divisor,
                remainder,
                divident,
            } => append_to_data!(Word::WB, divisor, remainder, divident),
            Instr::Not { register, value } => append_to_data!(Word::WC, register, value),
            Instr::And { register, value } => append_to_data!(Word::WD, register, value),
            Instr::Or { register, value } => append_to_data!(Word::WE, register, value),
            Instr::Xor { register, value } => append_to_data!(Word::WF, register, value),
        }
    }

    data
}

fn write_data(
    mut target: impl std::io::Write,
    data: impl IntoIterator<Item = Word>,
) -> Result<(), WritingError> {
    let mut data = data.into_iter();

    while let Some(high) = data.next() {
        let low = data.next().unwrap_or(Word::W0);
        let amount_written = target.write(&[((high as u8) << 4) | (low as u8)])?;
        snafu::ensure!(amount_written == 1, UnableToWriteEverythingSnafu);
    }

    Ok(())
}

#[cfg(test)]
mod testing {
    mod collect_data {
        use crate::tokens::{Instruction as Instr, values::test_prelude::*};
        use crate::writer::collect_data;

        fn template<const N: usize>(program: impl IntoIterator<Item = Instr>, expected: [Word; N]) {
            let data = std::hint::black_box(collect_data(program.into_iter()));
            assert_eq!(data, expected);
        }

        fn biop(instr: impl Fn(Register, Value) -> Instr) {
            for register in all_registers() {
                for value in all_registers() {
                    let instr = instr(register, Value::Register(value));
                    template([instr], [instr.id().into(), register.into(), value.into()]);
                }
                for value in all_literals() {
                    let instr = instr(register, Value::Literal(value));
                    template(
                        [instr],
                        [instr.id().into(), register.into(), Word::WF, value.into()],
                    );
                }
            }
        }

        #[test]
        fn halt() {
            template([Instr::Halt], [Word::W0]);
        }

        #[test]
        fn compare() {
            biop(|register, value| Instr::Compare { register, value })
        }

        #[test]
        fn signed_compare() {
            biop(|register, value| Instr::SignedCompare { register, value })
        }

        #[test]
        fn jump() {
            for address in all_addresses() {
                let instr = Instr::Jump { address };
                template([instr], [instr.id().into(), address.high, address.low]);
            }
        }

        #[test]
        fn skip() {
            const BITS: [bool; 2] = [false, true];

            for due_to_greater in BITS {
                for due_to_equal in BITS {
                    for due_to_overflow in BITS {
                        for due_to_zero_division in BITS {
                            let instr = Instr::Skip {
                                due_to_greater,
                                due_to_equal,
                                due_to_overflow,
                                due_to_zero_division,
                            };
                            template(
                                [instr],
                                [
                                    instr.id().into(),
                                    [
                                        due_to_greater,
                                        due_to_equal,
                                        due_to_overflow,
                                        due_to_zero_division,
                                    ]
                                    .into(),
                                ],
                            );
                        }
                    }
                }
            }
        }

        #[test]
        fn get() {
            for register in all_registers() {
                for address in all_addresses() {
                    let instr = Instr::Get { register, address };
                    template(
                        [instr],
                        [
                            instr.id().into(),
                            register.into(),
                            address.high,
                            address.low,
                        ],
                    );
                }
            }
        }

        #[test]
        fn set() {
            for register in all_registers() {
                for address in all_addresses() {
                    let instr = Instr::Set { register, address };
                    template(
                        [instr],
                        [
                            instr.id().into(),
                            register.into(),
                            address.high,
                            address.low,
                        ],
                    );
                }
            }
        }

        #[test]
        fn r#move() {
            biop(|register, value| Instr::Move { register, value })
        }

        #[test]
        fn add() {
            biop(|register, value| Instr::Add { register, value })
        }

        #[test]
        fn sub() {
            biop(|register, value| Instr::Sub { register, value })
        }

        #[test]
        fn mul() {
            biop(|register, value| Instr::Mul { register, value })
        }

        #[test]
        fn div() {
            for divisor in all_registers() {
                for remainder in all_registers() {
                    for divident in all_registers() {
                        let instr = Instr::Div {
                            divisor,
                            remainder,
                            divident: Value::Register(divident),
                        };
                        template(
                            [instr],
                            [
                                instr.id().into(),
                                divisor.into(),
                                remainder.into(),
                                divident.into(),
                            ],
                        );
                    }
                    for divident in all_literals() {
                        let instr = Instr::Div {
                            divisor,
                            remainder,
                            divident: Value::Literal(divident),
                        };
                        template(
                            [instr],
                            [
                                instr.id().into(),
                                divisor.into(),
                                remainder.into(),
                                Word::WF,
                                divident.into(),
                            ],
                        );
                    }
                }
            }
        }

        #[test]
        fn not() {
            biop(|register, value| Instr::Not { register, value })
        }

        #[test]
        fn and() {
            biop(|register, value| Instr::And { register, value })
        }

        #[test]
        fn or() {
            biop(|register, value| Instr::Or { register, value })
        }

        #[test]
        fn xor() {
            biop(|register, value| Instr::Xor { register, value })
        }
    }

    mod write_data {
        use snafu::ResultExt;

        use crate::tokens::values::test_prelude::*;
        use crate::writer::{WritingError, write_data};

        #[derive(Default, Debug, snafu::Snafu)]
        struct ErrorDump {
            pub errors: Vec<ErrorFragment>,
        }

        impl ErrorDump {
            fn add(&mut self, result: Result<(), ErrorFragment>) {
                if let Err(err) = result {
                    self.errors.push(err)
                }
            }

            fn dump(self) -> Result<(), Self> {
                self.errors.is_empty().then_some(()).ok_or(self)
            }
        }

        #[derive(Debug, snafu::Snafu)]
        struct ErrorFragment {
            #[snafu(source)]
            source: WritingError,

            seed: [u8; 32],
            data: Vec<Word>,
            expected: Vec<u8>,
        }

        fn one_write(
            data: impl IntoIterator<Item = Word>,
            expected: &[u8],
        ) -> Result<(), WritingError> {
            let mut target = Vec::with_capacity(expected.len());
            write_data(&mut target, data)?;
            assert_eq!(target, expected);
            Ok(())
        }

        fn for_specific_size(
            repeats: usize,
            data_size: std::ops::Range<usize>,
        ) -> Result<(), ErrorDump> {
            use rand::prelude::*;

            let uniform = rand::distr::Uniform::new(data_size.start, data_size.end)
                .expect("Invalid data_size");
            let mut data: Vec<Word> = Vec::with_capacity(data_size.end * 2);
            let mut expected: Vec<u8> = Vec::with_capacity(data_size.end);

            let mut single_iteration = || -> Result<(), ErrorFragment> {
                data.clear();
                expected.clear();

                let seed = rand::random();
                let mut rng = StdRng::from_seed(seed);

                let size = rng.sample(uniform) + 1;
                expected.resize_with(size, || rng.random());

                for &byte in &expected[..size] {
                    data.extend(crate::tokens::values::byte_to_pair(byte))
                }

                one_write(data.iter().copied(), &expected[..size]).with_context(|_| {
                    ErrorFragmentSnafu {
                        seed,
                        data: data.clone(),
                        expected: expected.clone(),
                    }
                })
            };

            let mut error_dump = ErrorDump::default();

            for _ in 0..repeats {
                error_dump.add(single_iteration())
            }

            error_dump.dump()
        }

        #[test]
        fn tiny_data() -> Result<(), ErrorDump> {
            for_specific_size(1, 0..1)?;
            for_specific_size(1, 1..2)?;
            for_specific_size(1, 2..3)?;
            for_specific_size(1, 3..4)?;
            for_specific_size(1, 4..5)?;
            Ok(())
        }

        #[test]
        fn small_data() -> Result<(), ErrorDump> {
            for_specific_size(200, 0..16)
        }

        #[test]
        fn medium_data() -> Result<(), ErrorDump> {
            for_specific_size(40_000, 16..64)
        }

        #[test]
        fn large_data() -> Result<(), ErrorDump> {
            for_specific_size(u16::MAX as usize, 64..256)
        }
    }
}
