pub trait FromStringPrivileged<TContext>: Sized {
    /// The associated error which can be returned from parsing.
    type Err;

    /// Parses a value from string with access to context provided by the caller.
    fn from_string_privileged(
        string: &str,
        context: &TContext,
    ) -> Result<Self, Self::Err>;
}
