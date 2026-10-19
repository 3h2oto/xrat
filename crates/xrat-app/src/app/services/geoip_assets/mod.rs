mod executor;
mod progress;
mod request;
mod summary;

#[cfg(test)]
mod tests;

use crate::app::AppError;
use crate::app::context::AppContext;
use crate::cli::GeoIpDownloadArgs;

use executor::execute_downloads;
use request::DownloadRequest;

mod transfer;
pub(crate) use transfer::run;

pub(crate) async fn download_assets(context: &AppContext) -> crate::app::Result<String> {
    let request = DownloadRequest {
        editions: edition::SUPPORTED_EDITIONS.to_vec(),
        mmdb_dir: resolve_mmdb_target_dir(context, None),
        force: true,
        url_template: context.app_config.mmdb.download_url.clone(),
        timeout_secs: context.app_config.mmdb.timeout_secs,
        quiet: true,
    };
    let summary = execute_downloads(&request).await;
    if !summary.failed.is_empty() {
        return Err(AppError::InvalidArgument("GeoIP update failed".into()));
    }
    Ok(summary.format())
}

pub(crate) mod edition;
fn resolve_mmdb_target_dir(
    context: &AppContext,
    override_dir: Option<&std::path::PathBuf>,
) -> std::path::PathBuf {
    override_dir.cloned().unwrap_or_else(|| {
        crate::app::paths::mmdb::resolve_mmdb_dir(&context.runtime_paths, &context.app_config)
    })
}
fn ensure_mmdb_target_dir(dir: &std::path::Path) -> crate::app::Result<()> {
    std::fs::create_dir_all(dir)?;
    Ok(())
}
fn mmdb_file_path(dir: &std::path::Path, edition: edition::MmdbEdition) -> std::path::PathBuf {
    dir.join(edition.file_name())
}
