use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use xrat_db::{ConfigRecord, ImportSource, ImportSummary, SubscriptionRecord};
use xrat_model::{ConfigId, ConfigRef, SubscriptionId};

use crate::app::services::{
    ConfigListRequest, ConfigListResult, DeleteOutcome, RestoreOutcome, ToggleOutcome,
};

pub(crate) const PAGE_LIMIT: i64 = 200;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum CatalogRequest {
    Profile {},
    Detail {
        config_id: ConfigId,
    },
    FindConfig {
        reference: ConfigRef,
    },
    FindSubscription {
        reference: String,
    },
    Purge {},
    List {
        filter: ConfigListRequest,
    },
    Config {
        reference: ConfigRef,
    },
    ResolveConfig {
        reference: ConfigRef,
    },
    Reference {
        config_id: ConfigId,
    },
    ResolveSubscription {
        reference: String,
    },
    Subscriptions {
        offset: u64,
        limit: u16,
    },
    Subscription {
        reference: String,
    },
    Import {
        contents: Vec<String>,
        source: ImportSource,
        name: Option<String>,
        headers: BTreeMap<String, String>,
    },
    Enable {
        reference: ConfigRef,
    },
    Disable {
        reference: ConfigRef,
    },
    Delete {
        reference: ConfigRef,
        hard: bool,
    },
    Restore {
        reference: ConfigRef,
    },
    DeleteMany {
        ids: Vec<ConfigId>,
        hard: bool,
    },
    RestoreMany {
        ids: Vec<ConfigId>,
    },
    DeleteSubscription {
        reference: String,
    },
    RenameSubscription {
        reference: String,
        name: String,
    },
    ExportPage {
        filter: ConfigListRequest,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum CatalogPayload {
    Profile {
        database_label: String,
        config_path: String,
    },
    Configs(ConfigListResult),
    Detail(Option<Box<crate::app::read_models::ConfigDetail>>),
    FoundConfig(Option<ConfigId>),
    FoundSubscription(Option<SubscriptionId>),
    Config(Box<ConfigRecord>),
    ConfigId(ConfigId),
    Reference(Option<String>),
    SubscriptionId(SubscriptionId),
    Subscriptions {
        total: u64,
        items: Vec<SubscriptionRecord>,
    },
    Subscription(Box<SubscriptionRecord>),
    Import(ImportSummary),
    Toggle {
        config: Box<ConfigRecord>,
        outcome: ToggleOutcome,
    },
    Delete {
        config: Box<ConfigRecord>,
        outcome: DeleteOutcome,
    },
    Restore {
        config: Box<ConfigRecord>,
        outcome: RestoreOutcome,
    },
    Changed(u64),
    ExportPage {
        total: i64,
        items: Vec<String>,
    },
}
