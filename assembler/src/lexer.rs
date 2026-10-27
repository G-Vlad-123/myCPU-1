use crate::tokens::{self, Instruction, InstructionId as IId, values};
use snafu::prelude::*;
use std::str::FromStr;

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
    IsNotUTF8,
}

#[derive(Debug, snafu::Snafu)]
pub enum ParseInstructionError {
    Empty,

    MalformedKeyword {
        string: String,
    },

    EarlyEOF,

    #[snafu(context(false))]
    ParseValue {
        #[snafu(source(from(exact)))]
        source: ParseValueError,
    },

    #[snafu(context(false))]
    ParseRegister {
        #[snafu(source(from(exact)))]
        source: ParseRegisterError,
    },

    ParseAddressHighWord {
        #[snafu(source)]
        source: ParseWordError,
    },

    ParseAddressLowWord {
        #[snafu(source)]
        source: ParseWordError,
    },
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

fn into_lines(_source: impl std::io::Read) -> impl Iterator<Item = Result<String, EncodeError>> {
    todo!("`lexer::into_lines`");
    #[allow(unreachable_code)]
    [].into_iter()
}

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
            IId::Skip => todo!("impl `FromStr::from_str match IId::Skip` for `Instruction`"),
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

        // ensure!(instruction.is_empty(), ...);

        Ok(ret)
    }
}

#[derive(Debug, snafu::Snafu)]
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

#[derive(Debug, snafu::Snafu)]
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

#[derive(Debug, snafu::Snafu)]
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

#[derive(Debug, snafu::Snafu)]
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
