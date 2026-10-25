use crate::tokens;

#[derive(Debug, snafu::Snafu)]
pub enum LexerError {}

pub fn tokenize_file(_source: impl std::io::Read) -> Result<tokens::Program, LexerError> {
    todo!("`lexer::tokenize_file`")
}
