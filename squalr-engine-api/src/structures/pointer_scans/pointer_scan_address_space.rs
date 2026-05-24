use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PointerScanAddressSpace {
    #[default]
    Auto,
    GameMemory,
    EmulatorMemory,
}

impl PointerScanAddressSpace {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::GameMemory => "Virtual memory",
            Self::EmulatorMemory => "Host memory",
        }
    }
}

impl fmt::Display for PointerScanAddressSpace {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::Auto => write!(formatter, "auto"),
            Self::GameMemory => write!(formatter, "virtual"),
            Self::EmulatorMemory => write!(formatter, "host"),
        }
    }
}

impl FromStr for PointerScanAddressSpace {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "auto" | "default" => Ok(Self::Auto),
            "game" | "guest" | "virtual" | "virtual-modules" | "virtualmodules" => Ok(Self::GameMemory),
            "emulator" | "host" | "raw" | "usermode" | "user-mode" => Ok(Self::EmulatorMemory),
            _ => Err(format!("Unsupported pointer scan address space: {}", input)),
        }
    }
}
