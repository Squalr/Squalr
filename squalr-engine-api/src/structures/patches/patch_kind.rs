use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum PatchKind {
    #[default]
    Code,
    NoOperation,
    SoftwareBreakpoint,
    Generic,
}

impl fmt::Display for PatchKind {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let patch_kind_label = match self {
            Self::Code => "code",
            Self::NoOperation => "no-operation",
            Self::SoftwareBreakpoint => "software-breakpoint",
            Self::Generic => "generic",
        };

        formatter.write_str(patch_kind_label)
    }
}

impl FromStr for PatchKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "code" => Ok(Self::Code),
            "nop" | "no-op" | "no-operation" | "no_operation" => Ok(Self::NoOperation),
            "software-breakpoint" | "software_breakpoint" | "soft-bp" | "int3" => Ok(Self::SoftwareBreakpoint),
            "generic" => Ok(Self::Generic),
            _ => Err(format!(
                "Unknown patch kind '{}'. Expected code, no-operation, software-breakpoint, or generic.",
                value
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PatchKind;

    #[test]
    fn patch_kind_parses_cli_aliases() {
        assert_eq!("code".parse::<PatchKind>(), Ok(PatchKind::Code));
        assert_eq!("nop".parse::<PatchKind>(), Ok(PatchKind::NoOperation));
        assert_eq!("int3".parse::<PatchKind>(), Ok(PatchKind::SoftwareBreakpoint));
        assert_eq!("generic".parse::<PatchKind>(), Ok(PatchKind::Generic));
    }
}
