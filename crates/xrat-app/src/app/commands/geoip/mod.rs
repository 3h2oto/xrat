use std::path::{Path, PathBuf};

use crate::app::context::AppContext;
use crate::app::paths::mmdb;
use crate::cli::{GeoIpAction, GeoIpArgs};

mod backend;
mod download;
mod edition;
mod lookup;
mod path;
mod status;
mod update;

pub async fn run(context: &AppContext, args: &GeoIpArgs) -> crate::app::Result<()> {
    match &args.action {
        GeoIpAction::Download(args) => download::run(context, args).await,
        GeoIpAction::Update(args) => update::run(context, args).await,
        GeoIpAction::Lookup(args) => lookup::run(context, args).await,
        GeoIpAction::Backend(args) => backend::run(context, args),
        GeoIpAction::Path(args) => path::run(context, args),
        GeoIpAction::Status(args) => status::run(context, args),
    }
}

fn resolve_mmdb_target_dir(context: &AppContext, output_override: Option<&PathBuf>) -> PathBuf {
    output_override
        .cloned()
        .unwrap_or_else(|| mmdb::resolve_mmdb_dir(&context.runtime_paths, &context.app_config))
}

fn ensure_mmdb_target_dir(dir: &Path) -> crate::app::Result<()> {
    std::fs::create_dir_all(dir)?;
    Ok(())
}

fn mmdb_file_path(dir: &Path, edition: edition::MmdbEdition) -> PathBuf {
    dir.join(edition.file_name())
}

fn mmdb_file_name(edition: edition::MmdbEdition) -> &'static str {
    edition.file_name()
}

const SUPPORTED_EDITIONS: [edition::MmdbEdition; 3] = edition::SUPPORTED_EDITIONS;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_default_target_dir_from_mmdb_config() {
        let context = test_context();

        assert_eq!(
            resolve_mmdb_target_dir(&context, None),
            context.runtime_paths.root_dir.join("mmdb")
        );
    }

    #[test]
    fn resolves_override_target_dir_verbatim() {
        let context = test_context();

        assert_eq!(
            resolve_mmdb_target_dir(&context, Some(&PathBuf::from("./tmp/mmdb"))),
            PathBuf::from("./tmp/mmdb")
        );
    }

    fn test_context() -> AppContext {
        tokio::runtime::Runtime::new()
            .expect("runtime should build")
            .block_on(crate::app::tests::TestAppBuilder::new("geoip").build())
    }
}
