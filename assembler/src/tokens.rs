#[derive(Default, Debug, Clone)]
pub struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    pub fn iter(&self) -> impl Clone + Iterator<Item = Instruction> {
        self.instructions.iter().cloned()
    }
}

impl<Instr: Into<Instruction>> FromIterator<Instr> for Program {
    fn from_iter<I: IntoIterator<Item = Instr>>(iter: I) -> Self {
        Program {
            instructions: iter.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Halt,

    Move {
        register: values::Register,
        value: values::Value,
    },

    Compare {
        register: values::Register,
        value: values::Value,
    },
    SignedCompare {
        register: values::Register,
        value: values::Value,
    },

    Jump {
        address: values::Address,
    },
    Skip {
        due_to_greater: bool,
        due_to_equal: bool,
        due_to_overflow: bool,
        due_to_zero_division: bool,
    },

    Get {
        register: values::Register,
        address: values::Address,
    },
    Set {
        register: values::Register,
        address: values::Address,
    },

    Add {
        register: values::Register,
        value: values::Value,
    },
    Sub {
        register: values::Register,
        value: values::Value,
    },
    Mul {
        register: values::Register,
        value: values::Value,
    },
    Div {
        divisor: values::Register,
        remainder: values::Register,
        divident: values::Value,
    },
    Not {
        register: values::Register,
        value: values::Value,
    },
    And {
        register: values::Register,
        value: values::Value,
    },
    Or {
        register: values::Register,
        value: values::Value,
    },
    Xor {
        register: values::Register,
        value: values::Value,
    },

    AddressName {
        name: String,
    },
    Const {
        constants: Vec<values::Literal>,
    },
}

impl Instruction {
    #[cfg(test)]
    pub fn id(&self) -> InstructionId {
        match self {
            Instruction::Halt => InstructionId::Halt,

            Instruction::Move { .. } => InstructionId::Move,

            Instruction::Compare { .. } => InstructionId::Compare,
            Instruction::SignedCompare { .. } => InstructionId::SignedCompare,

            Instruction::Jump { .. } => InstructionId::Jump,
            Instruction::Skip { .. } => InstructionId::Skip,

            Instruction::Get { .. } => InstructionId::Get,
            Instruction::Set { .. } => InstructionId::Set,

            Instruction::Add { .. } => InstructionId::Add,
            Instruction::Sub { .. } => InstructionId::Sub,
            Instruction::Mul { .. } => InstructionId::Mul,
            Instruction::Div { .. } => InstructionId::Div,
            Instruction::Not { .. } => InstructionId::Not,
            Instruction::And { .. } => InstructionId::And,
            Instruction::Or { .. } => InstructionId::Or,
            Instruction::Xor { .. } => InstructionId::Xor,

            Instruction::AddressName { .. } => InstructionId::AddressName,
            Instruction::Const { .. } => InstructionId::Const,
        }
    }

    /// Returns the amount of words this instruction will take up
    pub fn size(&self) -> u8 {
        fn add_one_if_literal(value: &values::Value, size: u8) -> u8 {
            match value {
                values::Value::Register(_) => size,
                values::Value::Literal(_) => size + 1,
            }
        }

        match self {
            Instruction::Halt => 1,

            Instruction::Move { value, .. } => add_one_if_literal(value, 3),

            Instruction::Compare { value, .. } => add_one_if_literal(value, 3),
            Instruction::SignedCompare { value, .. } => add_one_if_literal(value, 3),

            Instruction::Jump { .. } => 3,
            Instruction::Skip { .. } => 2,

            Instruction::Get { .. } => 4,
            Instruction::Set { .. } => 4,

            Instruction::Add { value, .. } => add_one_if_literal(value, 3),
            Instruction::Sub { value, .. } => add_one_if_literal(value, 3),
            Instruction::Mul { value, .. } => add_one_if_literal(value, 3),
            Instruction::Div { divident, .. } => add_one_if_literal(divident, 4),
            Instruction::Not { value, .. } => add_one_if_literal(value, 3),
            Instruction::And { value, .. } => add_one_if_literal(value, 3),
            Instruction::Or { value, .. } => add_one_if_literal(value, 3),
            Instruction::Xor { value, .. } => add_one_if_literal(value, 3),

            Instruction::AddressName { .. } => 0,
            Instruction::Const { constants } => constants.len() as u8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionId {
    Halt,

    Move,

    Compare,
    SignedCompare,

    Jump,
    Skip,

    Get,
    Set,

    Add,
    Sub,
    Mul,
    Div,
    Not,
    And,
    Or,
    Xor,

    AddressName,
    Const,
}

impl From<values::Word> for InstructionId {
    fn from(word: values::Word) -> Self {
        match word {
            values::Word::W0 => Self::Halt,

            values::Word::W1 => Self::Move,

            values::Word::W2 => Self::Compare,
            values::Word::W3 => Self::SignedCompare,

            values::Word::W4 => Self::Jump,
            values::Word::W5 => Self::Skip,

            values::Word::W6 => Self::Get,
            values::Word::W7 => Self::Set,

            values::Word::W8 => Self::Add,
            values::Word::W9 => Self::Sub,
            values::Word::WA => Self::Mul,
            values::Word::WB => Self::Div,
            values::Word::WC => Self::Not,
            values::Word::WD => Self::And,
            values::Word::WE => Self::Or,
            values::Word::WF => Self::Xor,
        }
    }
}

#[derive(Debug, snafu::Snafu)]
pub enum NotRealInstruction {
    #[snafu(display(
        "The given instruction is an address name (aka not a true assembly instruction)"
    ))]
    AddressName,

    #[snafu(display(
        "The given instruction is a constant declaration (aka not a true assembly instruction)"
    ))]
    Const,
}

impl TryFrom<InstructionId> for values::Word {
    type Error = NotRealInstruction;

    fn try_from(instruction_id: InstructionId) -> Result<Self, NotRealInstruction> {
        match instruction_id {
            InstructionId::Halt => Ok(Self::W0),

            InstructionId::Move => Ok(Self::W1),

            InstructionId::Compare => Ok(Self::W2),
            InstructionId::SignedCompare => Ok(Self::W3),

            InstructionId::Jump => Ok(Self::W4),
            InstructionId::Skip => Ok(Self::W5),

            InstructionId::Get => Ok(Self::W6),
            InstructionId::Set => Ok(Self::W7),

            InstructionId::Add => Ok(Self::W8),
            InstructionId::Sub => Ok(Self::W9),
            InstructionId::Mul => Ok(Self::WA),
            InstructionId::Div => Ok(Self::WB),
            InstructionId::Not => Ok(Self::WC),
            InstructionId::And => Ok(Self::WD),
            InstructionId::Or => Ok(Self::WE),
            InstructionId::Xor => Ok(Self::WF),

            InstructionId::AddressName => Err(NotRealInstruction::AddressName),
            InstructionId::Const => Err(NotRealInstruction::Const),
        }
    }
}

pub mod values;
