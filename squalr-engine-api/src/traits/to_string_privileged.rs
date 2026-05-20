use std::fmt::Formatter;

pub trait ToStringPrivileged<TContext> {
    fn to_string_privileged(
        &self,
        formatter: &mut Formatter<'_>,
        context: &TContext,
    ) -> std::fmt::Result;
}
