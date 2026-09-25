use serde::Serialize;

/// One validation finding. Beyond identifying the offending field, each
/// diagnostic explains why the value is rejected and how to repair it so users
/// can fix `config.toml` without reading source or guessing valid ranges.
/// diagnostic explains why the value is rejected and how to repair it so users
/// can fix `config.toml` without reading source or guessing valid ranges.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct Diagnostic {
    /// Config path such as `[runtime.socks].port`. Empty for structural or
    /// parse failures that are not tied to a single field.
    pub(crate) field: String,
    /// What is invalid about the current value.
    pub(crate) problem: String,
    /// Why the constraint matters.
    pub(crate) reason: String,
    /// How to fix it, including accepted values or ranges where relevant.
    pub(crate) fix: String,
}

impl Diagnostic {
    pub(crate) fn new(
        field: impl Into<String>,
        problem: impl Into<String>,
        reason: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            problem: problem.into(),
            reason: reason.into(),
            fix: fix.into(),
        }
    }

    /// A finding not tied to a specific config field (missing file, parse
    /// failure). Rendered without a leading field label.
    pub(crate) fn structural(
        problem: impl Into<String>,
        reason: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Self::new(String::new(), problem, reason, fix)
    }
}
