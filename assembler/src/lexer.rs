use crate::tokens::{self, Instruction};

#[derive(Debug, snafu::Snafu)]
pub enum LexerError {
    #[snafu(context(false))]
    Endode {
        #[snafu(source(from(exact)))]
        source: EncodeError,
    },

    #[snafu(context(false))]
    LexInstruction {
        #[snafu(source(from(exact)))]
        source: LexInstructionError,
    },
}

#[derive(Debug, snafu::Snafu)]
pub enum EncodeError {
    IsNotUTF8,
}

#[derive(Debug, snafu::Snafu)]
pub enum LexInstructionError {}

pub fn tokenize_file(source: impl std::io::Read) -> Result<tokens::Program, LexerError> {
    into_lines(source)
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

impl std::str::FromStr for Instruction {
    type Err = LexInstructionError;
    fn from_str(_s: &str) -> Result<Self, LexInstructionError> {
        todo!("impl `FromStr::from_str` for `Instruction`")
    }
}
