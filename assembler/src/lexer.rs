use crate::tokens::{self, Instruction};
use snafu::ResultExt;

#[derive(Debug, snafu::Snafu)]
pub enum LexerError {
    #[snafu(context(false))]
    Endode {
        #[snafu(source(from(exact)))]
        source: EncodeError,
    },

    #[snafu(context(false))]
    ParseInstruction {
        #[snafu(source(from(exact)))]
        source: ParseInstructionError,
    },
}

#[derive(Debug, snafu::Snafu)]
pub enum EncodeError {
    IsNotUTF8 {
        #[snafu(source)]
        source: std::string::FromUtf8Error,
    },

    UnableToContinueRead {
        #[snafu(source)]
        source: std::io::Error,
    },
}

#[derive(Debug, PartialEq, snafu::Snafu)]
pub enum ParseInstructionError {
    Empty,

    MalformedKeyword {
        string: String,
    },

    EarlyEOF,

    #[snafu(context(false))]
    ParseValue {
        #[snafu(source(from(exact)))]
        source: parsing::ParseValueError,
    },

    #[snafu(context(false))]
    ParseRegister {
        #[snafu(source(from(exact)))]
        source: parsing::ParseRegisterError,
    },

    ParseAddressHighWord {
        #[snafu(source)]
        source: parsing::ParseWordError,
    },

    ParseAddressLowWord {
        #[snafu(source)]
        source: parsing::ParseWordError,
    },

    TooManyArguments {
        arguments: Vec<String>,
    },

    #[snafu(context(false))]
    DoubleSkipArgument {
        #[snafu(source(from(exact)))]
        source: ParseInstructionSkipError,
    },
}

#[derive(Debug, PartialEq, snafu::Snafu)]
pub enum ParseInstructionSkipError {
    DoubleGreater,
    DoubleEqual,
    DoubleOverflow,
    DoubleZeroDivision,
    UnknownArgument { argument: String },
}

pub fn tokenize_file(source: impl std::io::Read) -> Result<tokens::Program, LexerError> {
    into_lines(source)
        .filter(|r| r.as_ref().map(|s| s.is_empty()).unwrap_or(true))
        .map(|r| {
            r.map(|s| s.parse::<Instruction>().map_err(Into::into))
                .map_err(Into::into)
                .flatten()
        })
        .collect()
}

fn into_lines(source: impl std::io::Read) -> impl Iterator<Item = Result<String, EncodeError>> {
    let mut current_line = Vec::new();
    let mut source = source.bytes();

    std::iter::from_fn(move || {
        loop {
            let Some(next) = source.next() else {
                if current_line.is_empty() {
                    return None;
                }

                return Some(
                    String::from_utf8(std::mem::take(&mut current_line)).context(IsNotUTF8Snafu),
                );
            };

            let byte = match next.context(UnableToContinueReadSnafu) {
                Ok(byte) => byte,
                Err(err) => return Some(Err(err)),
            };

            if byte == b'\n' {
                return Some(
                    String::from_utf8(std::mem::take(&mut current_line)).context(IsNotUTF8Snafu),
                );
            }

            current_line.push(byte);
        }
    })
}

mod parsing {
    use super::*;
    use crate::tokens::{Instruction, InstructionId as IId, values};
    use snafu::prelude::*;
    use std::str::FromStr;

    impl FromStr for Instruction {
        type Err = ParseInstructionError;
        fn from_str(s: &str) -> Result<Self, ParseInstructionError> {
            let mut instruction = s.split_whitespace();

            let keyword = instruction.next().context(EmptySnafu)?;

            fn next<'iter, 'str, T: FromStr<Err: Into<ParseInstructionError>>>(
                iter: &'iter mut impl Iterator<Item = &'str str>,
            ) -> Result<T, ParseInstructionError> {
                iter.next()
                    .context(EarlyEOFSnafu)?
                    .parse()
                    .map_err(Into::into)
            }

            fn address<'iter, 'str>(
                iter: &'iter mut impl Iterator<Item = &'str str>,
            ) -> Result<values::Address, ParseInstructionError> {
                Ok(values::Address {
                    high: iter
                        .next()
                        .context(EarlyEOFSnafu)?
                        .parse()
                        .context(ParseAddressHighWordSnafu)?,
                    low: iter
                        .next()
                        .context(EarlyEOFSnafu)?
                        .parse()
                        .context(ParseAddressLowWordSnafu)?,
                })
            }

            fn parse_skip<'iter, 'str>(
                iter: &'iter mut impl Iterator<Item = &'str str>,
            ) -> Result<Instruction, ParseInstructionSkipError> {
                let mut due_to_greater = false;
                let mut due_to_equal = false;
                let mut due_to_overflow = false;
                let mut due_to_zero_division = false;

                for arg in iter {
                    match arg {
                        "greater" => {
                            ensure!(!due_to_greater, DoubleGreaterSnafu);
                            due_to_greater = true;
                        }
                        "equal" => {
                            ensure!(!due_to_equal, DoubleEqualSnafu);
                            due_to_equal = true;
                        }
                        "overflow" => {
                            ensure!(!due_to_overflow, DoubleOverflowSnafu);
                            due_to_overflow = true;
                        }
                        "zero_division" => {
                            ensure!(!due_to_zero_division, DoubleZeroDivisionSnafu);
                            due_to_zero_division = true;
                        }
                        _ => ensure!(
                            false,
                            UnknownArgumentSnafu {
                                argument: arg.to_string()
                            }
                        ),
                    }
                }

                Ok(Instruction::Skip {
                    due_to_greater,
                    due_to_equal,
                    due_to_overflow,
                    due_to_zero_division,
                })
            }

            let id = match keyword {
                "hlt" => IId::Halt,
                "mov" => IId::Move,
                "cmp" => IId::Compare,
                "smp" => IId::SignedCompare,
                "jmp" => IId::Jump,
                "skp" => IId::Skip,
                "get" => IId::Get,
                "set" => IId::Set,
                "add" => IId::Add,
                "sub" => IId::Sub,
                "mul" => IId::Mul,
                "div" => IId::Div,
                "not" => IId::Not,
                "and" => IId::And,
                "or" => IId::Or,
                "xor" => IId::Xor,
                _ => {
                    return Err(ParseInstructionError::MalformedKeyword {
                        string: keyword.to_string(),
                    });
                }
            };

            let ret = match id {
                IId::Halt => Instruction::Halt,
                IId::Move => Instruction::Move {
                    register: next(&mut instruction)?,
                    value: next(&mut instruction)?,
                },
                IId::Compare => Instruction::Compare {
                    register: next(&mut instruction)?,
                    value: next(&mut instruction)?,
                },
                IId::SignedCompare => Instruction::SignedCompare {
                    register: next(&mut instruction)?,
                    value: next(&mut instruction)?,
                },
                IId::Jump => Instruction::Jump {
                    address: address(&mut instruction)?,
                },
                IId::Skip => parse_skip(&mut instruction)?,
                IId::Get => Instruction::Get {
                    register: next(&mut instruction)?,
                    address: address(&mut instruction)?,
                },
                IId::Set => Instruction::Set {
                    register: next(&mut instruction)?,
                    address: address(&mut instruction)?,
                },
                IId::Add => Instruction::Add {
                    register: next(&mut instruction)?,
                    value: next(&mut instruction)?,
                },
                IId::Sub => Instruction::Sub {
                    register: next(&mut instruction)?,
                    value: next(&mut instruction)?,
                },
                IId::Mul => Instruction::Mul {
                    register: next(&mut instruction)?,
                    value: next(&mut instruction)?,
                },
                IId::Div => Instruction::Div {
                    divisor: next(&mut instruction)?,
                    remainder: next(&mut instruction)?,
                    divident: next(&mut instruction)?,
                },
                IId::Not => Instruction::Not {
                    register: next(&mut instruction)?,
                    value: next(&mut instruction)?,
                },
                IId::And => Instruction::And {
                    register: next(&mut instruction)?,
                    value: next(&mut instruction)?,
                },
                IId::Or => Instruction::Or {
                    register: next(&mut instruction)?,
                    value: next(&mut instruction)?,
                },
                IId::Xor => Instruction::Xor {
                    register: next(&mut instruction)?,
                    value: next(&mut instruction)?,
                },
            };

            let remaining_arguments: Vec<String> = instruction.map(str::to_string).collect();
            ensure!(
                remaining_arguments.is_empty(),
                TooManyArgumentsSnafu {
                    arguments: remaining_arguments
                }
            );

            Ok(ret)
        }
    }

    #[derive(Debug, PartialEq, snafu::Snafu)]
    pub struct ParseWordError {
        string: String,
    }

    impl FromStr for values::Word {
        type Err = ParseWordError;

        fn from_str(s: &str) -> Result<Self, ParseWordError> {
            match s.trim() {
                "0" => Ok(Self::W0),
                "1" => Ok(Self::W1),
                "2" => Ok(Self::W2),
                "3" => Ok(Self::W3),
                "4" => Ok(Self::W4),
                "5" => Ok(Self::W5),
                "6" => Ok(Self::W6),
                "7" => Ok(Self::W7),
                "8" => Ok(Self::W8),
                "9" => Ok(Self::W9),
                "A" | "a" | "10" => Ok(Self::WA),
                "B" | "b" | "11" => Ok(Self::WB),
                "C" | "c" | "12" => Ok(Self::WC),
                "D" | "d" | "13" => Ok(Self::WD),
                "E" | "e" | "14" => Ok(Self::WE),
                "F" | "f" | "15" => Ok(Self::WF),
                _ => Err(ParseWordError {
                    string: s.to_string(),
                }),
            }
        }
    }

    #[derive(Debug, PartialEq, snafu::Snafu)]
    pub struct ParseRegisterError {
        string: String,
    }

    impl FromStr for values::Register {
        type Err = ParseRegisterError;

        fn from_str(s: &str) -> Result<Self, ParseRegisterError> {
            match s.trim() {
                "r0" | "R0" => Ok(Self::R0),
                "r1" | "R1" => Ok(Self::R1),
                "r2" | "R2" => Ok(Self::R2),
                "r3" | "R3" => Ok(Self::R3),
                "r4" | "R4" => Ok(Self::R4),
                "r5" | "R5" => Ok(Self::R5),
                "r6" | "R6" => Ok(Self::R6),
                "r7" | "R7" => Ok(Self::R7),
                "r8" | "R8" => Ok(Self::R8),
                "r9" | "R9" => Ok(Self::R9),
                "rA" | "RA" | "ra" | "Ra" | "r10" | "R10" => Ok(Self::RA),
                "rB" | "RB" | "rb" | "Rb" | "r11" | "R11" => Ok(Self::RB),
                "rC" | "RC" | "rc" | "Rc" | "r12" | "R12" => Ok(Self::RC),
                "rD" | "RD" | "rd" | "Rd" | "r13" | "R13" => Ok(Self::RD),
                "rE" | "RE" | "re" | "Re" | "r14" | "R14" => Ok(Self::RE),
                _ => Err(ParseRegisterError {
                    string: s.to_string(),
                }),
            }
        }
    }

    #[derive(Debug, PartialEq, snafu::Snafu)]
    #[snafu(context(false))]
    pub struct ParseLiteralError {
        #[snafu(source(from(exact)))]
        source: ParseWordError,
    }

    impl FromStr for values::Literal {
        type Err = ParseLiteralError;
        fn from_str(s: &str) -> Result<Self, ParseLiteralError> {
            s.parse::<values::Word>()
                .map(values::Literal)
                .map_err(Into::into)
        }
    }

    #[derive(Debug, PartialEq, snafu::Snafu)]
    pub struct ParseValueError {
        register_err: ParseRegisterError,
        literal_err: ParseLiteralError,
    }

    impl FromStr for values::Value {
        type Err = ParseValueError;
        fn from_str(s: &str) -> Result<Self, ParseValueError> {
            let literal_err = match s.parse::<values::Literal>() {
                Ok(literal) => return Ok(values::Value::Literal(literal)),
                Err(err) => err,
            };

            let register_err = match s.parse::<values::Register>() {
                Ok(register) => return Ok(values::Value::Register(register)),
                Err(err) => err,
            };

            Err(ParseValueError {
                register_err,
                literal_err,
            })
        }
    }
}

#[cfg(test)]
mod testing {
    mod into_lines {
        use crate::lexer::{EncodeError, into_lines};

        #[test]
        fn raw_text() -> Result<(), Vec<EncodeError>> {
            let mut strings = Vec::new();
            let mut errors = Vec::new();
            for result in into_lines(b"Hello my name is G. Vlad ... \n \t And I like to code!\n\nI also HATE hating because hating makes u hate everything and urself, and I hate hating myself because it makes me sad :'(".as_slice()) {
                match result {
                    Ok(string) => strings.push(string),
                    Err(error) => errors.push(error),
                }
            }

            assert_eq!(
                strings,
                [
                    "Hello my name is G. Vlad ... ",
                    " \t And I like to code!",
                    "",
                    "I also HATE hating because hating makes u hate everything and urself, and I hate hating myself because it makes me sad :'("
                ]
            );

            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    }

    mod parsing {
        use rand::seq::SliceRandom;
        use snafu::ResultExt;

        use crate::tokens::{
            Instruction as Instr,
            values::{Address, test_prelude::*},
        };

        #[derive(Debug, snafu::Snafu)]
        struct ErrorDump<T: std::str::FromStr<Err: 'static + std::error::Error>> {
            pub errors: Vec<ErrorFragment<T>>,
        }

        impl<T: std::str::FromStr<Err: 'static + std::error::Error>> ErrorDump<T> {
            fn new() -> Self {
                Self { errors: Vec::new() }
            }

            fn add(&mut self, result: Result<(), ErrorFragment<T>>) {
                if let Err(err) = result {
                    self.errors.push(err)
                }
            }

            fn dump(self) -> Result<(), Self> {
                self.errors.is_empty().then_some(()).ok_or(self)
            }
        }

        #[derive(snafu::Snafu)]
        struct ErrorFragment<T: std::str::FromStr<Err: 'static + std::error::Error>> {
            #[snafu(source)]
            source: T::Err,

            string: String,
            expected: T,
        }

        impl<T: std::fmt::Debug + std::str::FromStr<Err: 'static + std::error::Error>>
            std::fmt::Debug for ErrorFragment<T>
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("ErrorFragment")
                    .field("source", &self.source)
                    .field("string", &self.string)
                    .field("expected", &self.expected)
                    .finish()
            }
        }

        fn template<
            S: AsRef<str>,
            T: Copy
                + PartialEq
                + std::fmt::Debug
                + std::str::FromStr<Err: 'static + std::error::Error>,
        >(
            checks: impl IntoIterator<Item = (S, T)>,
        ) -> Result<(), ErrorDump<T>> {
            fn iteration<
                T: Copy
                    + PartialEq
                    + std::fmt::Debug
                    + std::str::FromStr<Err: 'static + std::error::Error>,
            >(
                string: impl AsRef<str>,
                expected: T,
            ) -> Result<(), ErrorFragment<T>> {
                let gotten_val =
                    string
                        .as_ref()
                        .parse::<T>()
                        .with_context(|_| ErrorFragmentSnafu {
                            string: string.as_ref().to_string(),
                            expected,
                        })?;
                assert_eq!(gotten_val, expected);
                Ok(())
            }

            let mut error_dump = ErrorDump::new();

            for (string, expected) in checks {
                error_dump.add(iteration(string, expected))
            }

            error_dump.dump()
        }

        #[rustfmt::skip]
        fn all_words() -> impl Iterator<Item = (&'static str, Word)> {
            use Word::*;

            [
                ("0", W0),
                ("1", W1),
                ("2", W2),
                ("3", W3),
                ("4", W4),
                ("5", W5),
                ("6", W6),
                ("7", W7),
                ("8", W8),
                ("9", W9),
                ("A", WA), ("a", WA), ("10", WA),
                ("B", WB), ("b", WB), ("11", WB),
                ("C", WC), ("c", WC), ("12", WC),
                ("D", WD), ("d", WD), ("13", WD),
                ("E", WE), ("e", WE), ("14", WE),
                ("F", WF), ("f", WF), ("15", WF),
            ]
            .into_iter()
        }

        #[rustfmt::skip]
        fn all_registers() -> impl Iterator<Item = (&'static str, Register)> {
            use Register::*;

            [
                ("R0", R0),
                ("R1", R1),
                ("R2", R2),
                ("R3", R3),
                ("R4", R4),
                ("R5", R5),
                ("R6", R6),
                ("R7", R7),
                ("R8", R8),
                ("R9", R9),
                ("RA", RA), ("Ra", RA), ("R10", RA),
                ("RB", RB), ("Rb", RB), ("R11", RB),
                ("RC", RC), ("Rc", RC), ("R12", RC),
                ("RD", RD), ("Rd", RD), ("R13", RD),
                ("RE", RE), ("Re", RE), ("R14", RE),

                ("r0", R0),
                ("r1", R1),
                ("r2", R2),
                ("r3", R3),
                ("r4", R4),
                ("r5", R5),
                ("r6", R6),
                ("r7", R7),
                ("r8", R8),
                ("r9", R9),
                ("rA", RA), ("ra", RA), ("r10", RA),
                ("rB", RB), ("rb", RB), ("r11", RB),
                ("rC", RC), ("rc", RC), ("r12", RC),
                ("rD", RD), ("rd", RD), ("r13", RD),
                ("rE", RE), ("re", RE), ("r14", RE),
            ]
            .into_iter()
        }

        fn all_literals() -> impl Iterator<Item = (&'static str, Literal)> {
            all_words().map(|(str, word)| (str, Literal(word)))
        }

        fn all_values() -> impl Iterator<Item = (&'static str, Value)> {
            all_literals()
                .map(|(str, literal)| (str, Value::Literal(literal)))
                .chain(all_registers().map(|(str, register)| (str, Value::Register(register))))
        }

        fn all_addresses() -> impl Iterator<Item = (String, Address)> {
            all_words()
                .map(|low| all_words().map(move |high| (high, low)))
                .flatten()
                .map(|(high, low)| {
                    (
                        format!("{} {}", high.0, low.0),
                        Address {
                            high: high.1,
                            low: low.1,
                        },
                    )
                })
        }

        #[test]
        fn word() {
            template(all_words())
                .map_err(|err| format!("{err:#?}"))
                .unwrap();
        }

        #[test]
        fn register() {
            template(all_registers())
                .map_err(|err| format!("{err:#?}"))
                .unwrap();
        }

        #[test]
        fn literal() {
            template(all_literals())
                .map_err(|err| format!("{err:#?}"))
                .unwrap();
        }

        #[test]
        fn value() {
            template(all_values())
                .map_err(|err| format!("{err:#?}"))
                .unwrap();
        }

        #[test]
        fn instruction_ok() {
            use std::iter;

            let halt = iter::once(("hlt".to_string(), Instr::Halt));

            fn biop(
                str_instr: &str,
                instr: impl Fn(Register, Value) -> Instr,
            ) -> impl Iterator<Item = (String, Instr)> {
                all_registers()
                    .map(|register| all_values().map(move |value| (register, value)))
                    .flatten()
                    .map(move |((str_reg, register), (str_val, value))| {
                        (
                            format!("{str_instr} {str_reg} {str_val}"),
                            instr(register, value),
                        )
                    })
            }

            let r#move = biop("mov", |register, value| Instr::Move { register, value });
            let compare = biop("cmp", |register, value| Instr::Compare { register, value });
            let signed_compare = biop("smp", |register, value| Instr::SignedCompare {
                register,
                value,
            });

            let jump = all_addresses()
                .map(|(str, address)| (format!("jmp {str}"), Instr::Jump { address }));

            // FIXME
            let skip = iter::once(("skp zero_div", Instr::Halt));

            let get = all_registers()
                .map(|register| all_addresses().map(move |address| (register, address)))
                .flatten()
                .map(|((str_reg, register), (str_addr, address))| {
                    (
                        format!("get {str_reg} {str_addr}"),
                        Instr::Get { register, address },
                    )
                });

            let set = all_registers()
                .map(|register| all_addresses().map(move |address| (register, address)))
                .flatten()
                .map(|((str_reg, register), (str_addr, address))| {
                    (
                        format!("set {str_reg} {str_addr}"),
                        Instr::Set { register, address },
                    )
                });

            let add = biop("add", |register, value| Instr::Add { register, value });
            let sub = biop("sub", |register, value| Instr::Sub { register, value });
            let mul = biop("mul", |register, value| Instr::Mul { register, value });
            let not = biop("not", |register, value| Instr::Not { register, value });
            let and = biop("and", |register, value| Instr::And { register, value });
            let or = biop("or", |register, value| Instr::Or { register, value });
            let xor = biop("xor", |register, value| Instr::Xor { register, value });

            let div = all_registers()
                .map(|divisor| all_registers().map(move |reminder| (divisor, reminder)))
                .flatten()
                .map(|(divisor, reminder)| {
                    all_values().map(move |divident| (divisor, reminder, divident))
                })
                .flatten()
                .map(move |(divisor, reminder, divident)| {
                    (
                        format!("div {} {} {}", divisor.0, reminder.0, divident.0),
                        Instr::Div {
                            divisor: divisor.1,
                            remainder: reminder.1,
                            divident: divident.1,
                        },
                    )
                });

            template(
                halt.chain(r#move)
                    .chain(compare)
                    .chain(signed_compare)
                    .chain(jump)
                    .chain(skip)
                    .chain(get)
                    .chain(set)
                    .chain(add)
                    .chain(sub)
                    .chain(mul)
                    .chain(div)
                    .chain(not)
                    .chain(and)
                    .chain(or)
                    .chain(xor),
            )
            .map_err(|err| format!("{err:#?}"))
            .unwrap();
        }

        #[test]
        fn instruction_err() {
            use crate::lexer::ParseInstructionError::*;

            let instr = "".parse::<Instr>();
            assert_eq!(instr, Err(Empty));

            for malformed_keyword in ["halt", "move", "hi", "no", "102", "❤️", "HLT", "Hlt"] {
                let instr = malformed_keyword.parse::<Instr>();
                assert_eq!(
                    instr,
                    Err(MalformedKeyword {
                        string: malformed_keyword.to_string()
                    })
                );
            }

            let instr = "mov".parse::<Instr>();
            assert_eq!(instr, Err(EarlyEOF));

            let instr = "mov r1 r2 r3 r4 r5".parse::<Instr>();
            assert_eq!(
                instr,
                Err(TooManyArguments {
                    arguments: vec![String::from("r3"), String::from("r4"), String::from("r5")]
                })
            );
        }
    }
}
