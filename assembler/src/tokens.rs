#[derive(Default, Debug, Clone)]
pub struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    pub fn new() -> Program {
        Self::default()
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction)
    }

    pub fn iter(&self) -> impl Iterator<Item = Instruction> {
        self.instructions.iter().copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        register: values::Register,
        value: values::Value,
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
}

impl Instruction {
    pub fn id(self) -> InstructionId {
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

impl From<InstructionId> for values::Word {
    fn from(instruction_id: InstructionId) -> Self {
        match instruction_id {
            InstructionId::Halt => Self::W0,

            InstructionId::Move => Self::W1,

            InstructionId::Compare => Self::W2,
            InstructionId::SignedCompare => Self::W3,

            InstructionId::Jump => Self::W4,
            InstructionId::Skip => Self::W5,

            InstructionId::Get => Self::W6,
            InstructionId::Set => Self::W7,

            InstructionId::Add => Self::W8,
            InstructionId::Sub => Self::W9,
            InstructionId::Mul => Self::WA,
            InstructionId::Div => Self::WB,
            InstructionId::Not => Self::WC,
            InstructionId::And => Self::WD,
            InstructionId::Or => Self::WE,
            InstructionId::Xor => Self::WF,
        }
    }
}

pub mod values;
