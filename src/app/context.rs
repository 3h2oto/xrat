mod paths;

use crate::app::config::AppConfig;
use crate::cli;
use crate::db::Database;

pub use paths::{RuntimePaths, resolve_config_path};

#[derive(Clone)]
pub struct AppContext {
    pub db: Database,
    pub app_config: AppConfig,
    pub runtime_paths: RuntimePaths,
}

impl AppContext {
    pub async fn build(args: &cli::Cli) -> crate::app::Result<Self> {
        let (runtime_paths, app_config) = paths::resolve_runtime(args)?;
        let db = Database::connect(&runtime_paths.database_config).await?;
        let context = Self {
            db,
            app_config,
            runtime_paths,
        };
        crate::app::runtime_service::log_retention::cleanup(&context).await;
        Ok(context)
    }

    /// Application services wired from this context.
    ///
    /// Cheap to build: it only wraps clones of the database handle and default
    /// ports. Call it at the start of a handler rather than caching a copy.
    pub fn services(&self) -> crate::app::services::AppServices {
        crate::app::services::AppServices::from_context(self)
    }
}

#[cfg(test)]
mod tests;
