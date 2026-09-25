use crate::app::config::{RoutingSettings, ServerSettings};
use crate::app::services::AppServices;
use crate::app::services::proxy_pac::PacRules;
use crate::db::Database;

#[derive(Clone)]
pub struct ServerState {
    pub db: Database,
    pub services: AppServices,
    pub api_key: Option<String>,
    pub pac_enabled: bool,
    pub pac_allowed_hosts: Vec<String>,
    pub pac_rules: PacRules,
}

impl ServerState {
    pub fn from_settings(
        db: Database,
        settings: &ServerSettings,
        routing: &RoutingSettings,
    ) -> crate::app::Result<ServerState> {
        let api_key = settings
            .key
            .as_ref()
            .map(|secret| secret.resolve())
            .transpose()?;
        let services = crate::app::services::AppServices::from_database(db.clone());
        Ok(Self {
            db,
            services,
            api_key,
            pac_enabled: settings.pac_enabled,
            pac_allowed_hosts: settings.pac_allowed_hosts.clone(),
            pac_rules: PacRules::from_routing(routing),
        })
    }

    /// Build a minimal state for tests, deriving services from the database.
    #[cfg(test)]
    pub fn for_test(db: Database, api_key: Option<String>) -> Self {
        Self {
            services: crate::app::services::AppServices::from_database(db.clone()),
            db,
            api_key,
            pac_enabled: true,
            pac_allowed_hosts: crate::app::config::defaults::DEFAULT_SERVER_PAC_ALLOWED_HOSTS
                .iter()
                .map(|host| host.to_string())
                .collect(),
            pac_rules: PacRules::default(),
        }
    }
}
