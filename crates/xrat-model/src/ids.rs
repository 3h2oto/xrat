use std::fmt;

use serde::{Deserialize, Serialize};

/// Internal database id for a stored config row.
///
/// Distinguishes config ids from subscription ids at compile time so argument
/// swaps are caught early. Transparent to `i64` for serde and SQL.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct ConfigId(pub i64);

/// Internal database id for a stored subscription row.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct SubscriptionId(pub i64);

/// Unresolved user-supplied config token, such as a numeric id or ref prefix.
///
/// Passed to resolution, which returns a concrete [`ConfigId`]. Encodes the
/// resolved/unresolved distinction in the type system.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConfigRef(pub String);

macro_rules! id_display {
    ($name:ident) => {
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }
    };
}

id_display!(ConfigId);
id_display!(SubscriptionId);
id_display!(ConfigRef);

macro_rules! id_from_primitive {
    ($name:ident, $prim:ty) => {
        impl From<$prim> for $name {
            fn from(value: $prim) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $prim {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl AsRef<$prim> for $name {
            fn as_ref(&self) -> &$prim {
                &self.0
            }
        }
    };
}

id_from_primitive!(ConfigId, i64);
id_from_primitive!(SubscriptionId, i64);

impl From<String> for ConfigRef {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for ConfigRef {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<ConfigRef> for String {
    fn from(value: ConfigRef) -> Self {
        value.0
    }
}

impl AsRef<str> for ConfigRef {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_preserve_numeric_json_contracts() {
        assert_eq!(serde_json::to_value(ConfigId(7)).unwrap(), 7);
        assert_eq!(serde_json::to_value(SubscriptionId(9)).unwrap(), 9);
        assert_eq!(serde_json::from_str::<ConfigId>("7").unwrap(), ConfigId(7));
        assert_eq!(
            serde_json::from_str::<SubscriptionId>("9").unwrap(),
            SubscriptionId(9)
        );
        assert_eq!(serde_json::to_value(Some(ConfigId(7))).unwrap(), 7);
        assert_eq!(
            serde_json::to_value(None::<SubscriptionId>).unwrap(),
            serde_json::Value::Null
        );
        assert!(serde_json::from_str::<ConfigId>("\"7\"").is_err());
    }

    #[test]
    fn refs_preserve_string_json_contracts() {
        let token = ConfigRef::from("1234abcd");
        assert_eq!(serde_json::to_value(&token).unwrap(), "1234abcd");
        assert_eq!(
            serde_json::from_str::<ConfigRef>("\"1234abcd\"").unwrap(),
            token
        );
    }

    #[test]
    fn id_display_preserves_numeric_formatting() {
        assert_eq!(format!("{:09}", ConfigId(7)), "000000007");
        assert_eq!(format!("{:>4}", SubscriptionId(9)), "   9");
    }
}
