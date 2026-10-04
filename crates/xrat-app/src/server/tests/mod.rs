use crate::server::ServerState;
use xrat_db::{ConnectionTestInsert, Database, DatabaseConnectionConfig, ImportSource, SourceKind};
use xrat_model::{Node, Protocol};

pub(super) mod bind_addr;
pub(super) mod routes_b64;
pub(super) mod routes_configs;
pub(super) mod routes_health;
pub(super) mod routes_json;
pub(super) mod routes_pac;
mod shutdown;

mod fixtures;
pub(super) use fixtures::multi_config_state;
pub(super) use fixtures::populated_state;
pub(super) use fixtures::test_node;
