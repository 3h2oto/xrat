/// Result of resolving a user-supplied ref prefix to an internal id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefMatch<Id> {
    /// No row matched the prefix.
    None,
    /// Exactly one row matched; carries its internal id.
    Unique(Id),
    /// More than one row matched the prefix.
    Ambiguous,
}
