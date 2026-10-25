use crate::tokens;

#[derive(Debug, snafu::Snafu)]
pub enum WritingError {}

pub fn write(
    _target: &mut impl std::io::Write,
    _program: impl Iterator<Item = tokens::Instruction>,
) -> Result<(), WritingError> {
    todo!("`writer::write`")
}
