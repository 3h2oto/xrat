use super::paths::{self, RuntimePaths};
use crate::app::config::AppConfig;
use crate::cli;
use xrat_db::Database;

#[derive(Clone)]
pub struct AppContext {
    pub db: Database,
    pub app_config: AppConfig,
    pub runtime_paths: RuntimePaths,
}

impl AppContext {
    /// Build from parsed CLI arguments, resolving paths from disk.
    pub async fn build(args: &cli::Cli) -> crate::app::Result<Self> {
        let (runtime_paths, app_config) = paths::resolve_runtime(args)?;
        build_app_context(runtime_paths, app_config).await
    }

    /// Build from already-resolved paths and config, without touching the CLI.
    ///
    /// This is the composition seam used by hosts and tests that construct an
    /// application context directly instead of parsing `cli::Cli`.
    pub async fn from_parts(
        runtime_paths: RuntimePaths,
        app_config: AppConfig,
    ) -> crate::app::Result<Self> {
        build_app_context(runtime_paths, app_config).await
    }

    /// Application services wired from this context.
    ///
    /// Cheap to build: it only wraps clones of the database handle and default
    /// ports. Call it at the start of a handler rather than caching a copy.
    pub fn services(&self) -> crate::app::services::AppServices {
        crate::app::services::AppServices::from_context(self)
    }
}

/// Build an [`AppContext`] from resolved runtime paths and app configuration.
///
/// Centralizes database connection and startup side effects (log retention) so
/// every entry point composes the context the same way.
pub async fn build_app_context(
    runtime_paths: RuntimePaths,
    app_config: AppConfig,
) -> crate::app::Result<AppContext> {
    let started = std::time::Instant::now();
    let db = Database::connect(&runtime_paths.database_config).await?;
    let context = AppContext {
        db,
        app_config,
        runtime_paths,
    };
    crate::app::runtime_service::log_retention::cleanup(&context).await;
    tracing::debug!(
        elapsed_ms = started.elapsed().as_millis(),
        "app context ready"
    );
    Ok(context)
}
