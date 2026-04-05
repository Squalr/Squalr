use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageRetrievalMode {
    #[default]
    FromSettings,
    FromUserMode,
    FromNonModules,
    FromModules,
    FromVirtualModules,
}

impl FromStr for PageRetrievalMode {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.to_ascii_lowercase().as_str() {
            "settings" | "fromsettings" => Ok(Self::FromSettings),
            "usermode" | "host" | "fromusermode" => Ok(Self::FromUserMode),
            "nonmodules" | "fromnonmodules" => Ok(Self::FromNonModules),
            "modules" | "frommodules" => Ok(Self::FromModules),
            "virtual" | "guest" | "virtualmodules" | "fromvirtualmodules" => Ok(Self::FromVirtualModules),
            _ => Err(format!("Unknown page retrieval mode: '{}'", string)),
        }
    }
}
