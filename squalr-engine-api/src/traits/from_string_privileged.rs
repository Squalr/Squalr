use crate::registries::registries::Registries;

pub trait FromStringPrivileged: Sized {
    /// The associated error which can be returned from parsing.
    type Err;

    /// Parses a value from string, with access to all engine registries.
    fn from_string_privileged(
        string: &str,
        registries: &Registries,
    ) -> Result<Self, Self::Err>;
}
