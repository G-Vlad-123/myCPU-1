use crate::tokens::{
    Instruction as Instr,
    values::{AppendTo, Word},
};

#[derive(Debug, snafu::Snafu)]
pub enum WritingError {}

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

fn write_data(_target: impl std::io::Write, _data: Vec<Word>) -> Result<(), WritingError> {
    todo!("`writer::write_data`")
}

#[cfg(test)]
mod testing {
    #[test]
    fn collect_data() {
        todo!("test `writer::testing::collect_data`")
    }

    #[test]
    fn write_data() {
        todo!("test `writer::testing::write_data`")
    }
}
