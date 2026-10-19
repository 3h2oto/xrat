use super::Engine;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    Bulk {
        operation: BulkOperation,
        ids: Vec<i64>,
    },
    Import {
        input: String,
        name: Option<String>,
    },
    Add {
        input: String,
    },
    Enable {
        id: i64,
    },
    Disable {
        id: i64,
    },
    Delete {
        id: i64,
        hard: bool,
    },
    Restore {
        id: i64,
    },
    DeleteSubscription {
        id: i64,
    },
    RenameSubscription {
        id: i64,
        name: String,
    },
    Connect {
        id: i64,
    },
    Disconnect,
    Refresh {
        id: Option<i64>,
    },
    Test {
        id: Option<i64>,
    },
    Rotate,
    Scan {
        ips: Vec<String>,
        port: u16,
        timeout_ms: u64,
    },
    Parse {
        input: String,
    },
    Purge,
    ClearEvents,
    InstallEngine {
        engine: Engine,
    },
    Tun {
        enabled: bool,
    },
    Validate,
    Upgrade {
        version: Option<String>,
    },
    GeoIpUpdate,
    GeoIpLookup {
        address: String,
    },
    Rotation {
        enabled: bool,
    },
}

impl Action {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Bulk { .. } => "bulk",
            Self::Import { .. } => "import",
            Self::Add { .. } => "add",
            Self::Enable { .. } => "enable",
            Self::Disable { .. } => "disable",
            Self::Delete { .. } => "delete",
            Self::Restore { .. } => "restore",
            Self::DeleteSubscription { .. } => "delete_subscription",
            Self::RenameSubscription { .. } => "rename_subscription",
            Self::Connect { .. } => "connect",
            Self::Disconnect => "disconnect",
            Self::Refresh { .. } => "refresh",
            Self::Test { .. } => "test",
            Self::Rotate => "rotate",
            Self::Scan { .. } => "scan",
            Self::Parse { .. } => "parse",
            Self::Purge => "purge",
            Self::ClearEvents => "clear_events",
            Self::InstallEngine { .. } => "install_engine",
            Self::Tun { .. } => "tun",
            Self::Validate => "validate",
            Self::Upgrade { .. } => "upgrade",
            Self::GeoIpUpdate => "geoip_update",
            Self::GeoIpLookup { .. } => "geoip_lookup",
            Self::Rotation { .. } => "rotation",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkOperation {
    Enable,
    Disable,
    Test,
    Delete,
    Restore,
    RefreshSubscriptions,
    DeleteSubscriptions,
}

impl BulkOperation {
    pub fn action(self, id: i64) -> Action {
        match self {
            Self::Enable => Action::Enable { id },
            Self::Disable => Action::Disable { id },
            Self::Test => Action::Test { id: Some(id) },
            Self::Delete => Action::Delete { id, hard: false },
            Self::Restore => Action::Restore { id },
            Self::RefreshSubscriptions => Action::Refresh { id: Some(id) },
            Self::DeleteSubscriptions => Action::DeleteSubscription { id },
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Enable => "Enable configurations",
            Self::Disable => "Disable configurations",
            Self::Test => "Test configurations",
            Self::Delete => "Delete configurations",
            Self::Restore => "Restore configurations",
            Self::RefreshSubscriptions => "Refresh subscriptions",
            Self::DeleteSubscriptions => "Delete subscriptions",
        }
    }
}
