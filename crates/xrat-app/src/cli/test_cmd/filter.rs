use super::TestArgs;
use xrat_db::ConfigListFilter;
use xrat_model::SubscriptionId;

impl TestArgs {
    pub fn config_filter(&self, subscription_id: Option<SubscriptionId>) -> ConfigListFilter {
        ConfigListFilter {
            only_enabled: self.enabled_only,
            only_active: self.active_only,
            only_deleted: false,
            include_deleted: false,
            subscription_id,
            protocol: None,
        }
    }
}
