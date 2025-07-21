use crate::registries::registries::Registries;
use std::fmt::Formatter;

pub trait ToStringPrivileged {
    fn to_string_privileged(
        &self,
        formatter: &mut Formatter<'_>,
        registries: &Registries,
    ) -> std::fmt::Result;
}
