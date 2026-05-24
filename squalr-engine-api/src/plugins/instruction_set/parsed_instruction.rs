use crate::plugins::instruction_set::InstructionDecorators;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstructionMemoryOperandSize {
    Byte,
    Word,
    Dword,
    Fword,
    Qword,
    Tbyte,
    Xmmword,
    Ymmword,
    Zmmword,
}

impl InstructionMemoryOperandSize {
    pub fn size_in_bytes(&self) -> Option<usize> {
        match self {
            Self::Byte => Some(1),
            Self::Word => Some(2),
            Self::Dword => Some(4),
            Self::Fword => Some(6),
            Self::Qword => Some(8),
            Self::Tbyte => Some(10),
            Self::Xmmword => Some(16),
            Self::Ymmword => Some(32),
            Self::Zmmword => Some(64),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstructionMemoryOperand {
    size: Option<InstructionMemoryOperandSize>,
    segment_override: Option<String>,
    expression_text: String,
    is_broadcast: bool,
}

impl InstructionMemoryOperand {
    pub fn new(
        size: Option<InstructionMemoryOperandSize>,
        expression_text: impl Into<String>,
    ) -> Self {
        Self::with_segment_override(size, None::<String>, expression_text)
    }

    pub fn with_segment_override(
        size: Option<InstructionMemoryOperandSize>,
        segment_override: Option<impl Into<String>>,
        expression_text: impl Into<String>,
    ) -> Self {
        Self {
            size,
            segment_override: segment_override.map(|segment_override| segment_override.into()),
            expression_text: expression_text.into(),
            is_broadcast: false,
        }
    }

    pub fn with_metadata(
        size: Option<InstructionMemoryOperandSize>,
        segment_override: Option<impl Into<String>>,
        expression_text: impl Into<String>,
        is_broadcast: bool,
    ) -> Self {
        Self {
            size,
            segment_override: segment_override.map(|segment_override| segment_override.into()),
            expression_text: expression_text.into(),
            is_broadcast,
        }
    }

    pub fn size(&self) -> Option<InstructionMemoryOperandSize> {
        self.size
    }

    pub fn segment_override(&self) -> Option<&str> {
        self.segment_override.as_deref()
    }

    pub fn expression_text(&self) -> &str {
        &self.expression_text
    }

    pub fn is_broadcast(&self) -> bool {
        self.is_broadcast
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstructionOperand {
    Identifier(String),
    Immediate(i128),
    Memory(InstructionMemoryOperand),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedInstruction {
    mnemonic: String,
    operands: Vec<InstructionOperand>,
    decorators: InstructionDecorators,
    source_text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedInstructionSequence {
    instructions: Vec<ParsedInstruction>,
    label_instruction_indices: HashMap<String, usize>,
}

impl ParsedInstruction {
    pub fn new(
        mnemonic: impl Into<String>,
        operands: Vec<InstructionOperand>,
        decorators: InstructionDecorators,
        source_text: impl Into<String>,
    ) -> Self {
        Self {
            mnemonic: mnemonic.into(),
            operands,
            decorators,
            source_text: source_text.into(),
        }
    }

    pub fn mnemonic(&self) -> &str {
        &self.mnemonic
    }

    pub fn operands(&self) -> &[InstructionOperand] {
        &self.operands
    }

    pub fn decorators(&self) -> &InstructionDecorators {
        &self.decorators
    }

    pub fn source_text(&self) -> &str {
        &self.source_text
    }
}

impl ParsedInstructionSequence {
    pub fn new(
        instructions: Vec<ParsedInstruction>,
        label_instruction_indices: HashMap<String, usize>,
    ) -> Self {
        Self {
            instructions,
            label_instruction_indices,
        }
    }

    pub fn instructions(&self) -> &[ParsedInstruction] {
        &self.instructions
    }

    pub fn label_instruction_indices(&self) -> &HashMap<String, usize> {
        &self.label_instruction_indices
    }
}
