use crate::tokens;

#[derive(Debug, snafu::Snafu)]
pub enum LexerError {}

pub fn tokenize_file(_file: std::fs::File) -> Result<tokens::Program, LexerError> {
    todo!("`lexer::tokenize_file`")
}
