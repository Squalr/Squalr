use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructViewerContainerMode {
    Element,
    Array,
}

impl StructViewerContainerMode {
    pub const ALL: [Self; 2] = [Self::Element, Self::Array];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Element => "Element",
            Self::Array => "Array",
        }
    }
}

impl FromStr for StructViewerContainerMode {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.trim() {
            "Element" => Ok(Self::Element),
            "Array" => Ok(Self::Array),
            invalid_mode => Err(format!("Invalid struct viewer container mode: {}", invalid_mode)),
        }
    }
}
