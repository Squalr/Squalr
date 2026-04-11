#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arm64RegisterWidth {
    W,
    X,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arm64RegisterKind {
    General,
    StackPointer,
    Zero,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Arm64Register {
    index: u8,
    width: Arm64RegisterWidth,
    kind: Arm64RegisterKind,
}

impl Arm64Register {
    pub fn new(
        index: u8,
        width: Arm64RegisterWidth,
        kind: Arm64RegisterKind,
    ) -> Self {
        Self { index, width, kind }
    }

    pub fn index(&self) -> u8 {
        self.index
    }

    pub fn width(&self) -> Arm64RegisterWidth {
        self.width
    }

    pub fn kind(&self) -> Arm64RegisterKind {
        self.kind
    }
}

pub fn parse_arm64_register_name(register_name: &str) -> Option<Arm64Register> {
    let normalized_register_name = register_name.trim().to_ascii_lowercase();

    match normalized_register_name.as_str() {
        "sp" => Some(Arm64Register::new(31, Arm64RegisterWidth::X, Arm64RegisterKind::StackPointer)),
        "wsp" => Some(Arm64Register::new(31, Arm64RegisterWidth::W, Arm64RegisterKind::StackPointer)),
        "xzr" => Some(Arm64Register::new(31, Arm64RegisterWidth::X, Arm64RegisterKind::Zero)),
        "wzr" => Some(Arm64Register::new(31, Arm64RegisterWidth::W, Arm64RegisterKind::Zero)),
        _ => parse_prefixed_arm64_register(&normalized_register_name),
    }
}

pub fn format_arm64_general_register(
    register_width: Arm64RegisterWidth,
    register_index: u8,
) -> String {
    if register_index == 31 {
        return match register_width {
            Arm64RegisterWidth::W => String::from("wzr"),
            Arm64RegisterWidth::X => String::from("xzr"),
        };
    }

    match register_width {
        Arm64RegisterWidth::W => format!("w{}", register_index),
        Arm64RegisterWidth::X => format!("x{}", register_index),
    }
}

pub fn format_arm64_base_register(register_index: u8) -> String {
    if register_index == 31 {
        String::from("sp")
    } else {
        format!("x{}", register_index)
    }
}

fn parse_prefixed_arm64_register(register_name: &str) -> Option<Arm64Register> {
    let (register_index_text, register_width) = if let Some(register_index_text) = register_name.strip_prefix('w') {
        (register_index_text, Arm64RegisterWidth::W)
    } else if let Some(register_index_text) = register_name.strip_prefix('x') {
        (register_index_text, Arm64RegisterWidth::X)
    } else {
        return None;
    };

    register_index_text
        .parse::<u8>()
        .ok()
        .filter(|register_index| *register_index <= 30)
        .map(|register_index| Arm64Register::new(register_index, register_width, Arm64RegisterKind::General))
}
