use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum DebuggerDataBreakpointAccess {
    Read,
    Write,
    ReadWrite,
}

impl DebuggerDataBreakpointAccess {
    pub fn get_cli_label(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::ReadWrite => "read-write",
        }
    }
}

impl FromStr for DebuggerDataBreakpointAccess {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "read" | "r" => Ok(Self::Read),
            "write" | "w" => Ok(Self::Write),
            "read-write" | "readwrite" | "rw" | "access" => Ok(Self::ReadWrite),
            _ => Err(format!("Unknown debugger data breakpoint access '{}'.", value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DebuggerDataBreakpointAccess;

    #[test]
    fn parses_access_trace_shorthands() {
        assert_eq!("r".parse::<DebuggerDataBreakpointAccess>(), Ok(DebuggerDataBreakpointAccess::Read));
        assert_eq!("w".parse::<DebuggerDataBreakpointAccess>(), Ok(DebuggerDataBreakpointAccess::Write));
        assert_eq!("rw".parse::<DebuggerDataBreakpointAccess>(), Ok(DebuggerDataBreakpointAccess::ReadWrite));
        assert_eq!("access".parse::<DebuggerDataBreakpointAccess>(), Ok(DebuggerDataBreakpointAccess::ReadWrite));
    }
}
