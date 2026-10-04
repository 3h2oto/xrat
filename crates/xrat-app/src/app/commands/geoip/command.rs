use super::*;

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

pub(super) fn resolve_mmdb_target_dir(
    context: &AppContext,
    output_override: Option<&PathBuf>,
) -> PathBuf {
    output_override
        .cloned()
        .unwrap_or_else(|| mmdb::resolve_mmdb_dir(&context.runtime_paths, &context.app_config))
}

pub(super) fn ensure_mmdb_target_dir(dir: &Path) -> crate::app::Result<()> {
    std::fs::create_dir_all(dir)?;
    Ok(())
}

pub(super) fn mmdb_file_path(dir: &Path, edition: edition::MmdbEdition) -> PathBuf {
    dir.join(edition.file_name())
}

pub(super) fn mmdb_file_name(edition: edition::MmdbEdition) -> &'static str {
    edition.file_name()
}

pub(super) const SUPPORTED_EDITIONS: [edition::MmdbEdition; 3] = edition::SUPPORTED_EDITIONS;
