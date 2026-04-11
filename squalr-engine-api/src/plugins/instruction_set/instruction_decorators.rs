#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstructionRoundingControl {
    RoundToNearest,
    RoundDown,
    RoundUp,
    RoundTowardZero,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InstructionDecorators {
    op_mask_register_name: Option<String>,
    zeroing_masking: bool,
    suppress_all_exceptions: bool,
    rounding_control: Option<InstructionRoundingControl>,
}

impl InstructionDecorators {
    pub fn new(
        op_mask_register_name: Option<impl Into<String>>,
        zeroing_masking: bool,
        suppress_all_exceptions: bool,
        rounding_control: Option<InstructionRoundingControl>,
    ) -> Self {
        Self {
            op_mask_register_name: op_mask_register_name.map(|op_mask_register_name| op_mask_register_name.into()),
            zeroing_masking,
            suppress_all_exceptions,
            rounding_control,
        }
    }

    pub fn op_mask_register_name(&self) -> Option<&str> {
        self.op_mask_register_name.as_deref()
    }

    pub fn zeroing_masking(&self) -> bool {
        self.zeroing_masking
    }

    pub fn suppress_all_exceptions(&self) -> bool {
        self.suppress_all_exceptions
    }

    pub fn rounding_control(&self) -> Option<InstructionRoundingControl> {
        self.rounding_control
    }

    pub fn has_any(&self) -> bool {
        self.op_mask_register_name.is_some() || self.zeroing_masking || self.suppress_all_exceptions || self.rounding_control.is_some()
    }
}
