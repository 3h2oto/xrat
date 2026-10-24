use crate::app::services::{
    cores,
    machine_service::{MachineProfile, scm},
};
use crate::app::{AppError, Result};
use crate::cli::{InstallArgs, InstallCore};
use std::io::Write;
use windows_sys::Win32::System::Services::SERVICE_STOPPED;

pub(super) async fn install(args: &InstallArgs) -> Result<()> {
    install_inner(args).await.map_err(|error| match error {
        AppError::Io(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            AppError::InvalidArgument(format!(
                "machine core provisioning requires an elevated administrator shell: {error}"
            ))
        }
        error => error,
    })
}

async fn install_inner(args: &InstallArgs) -> Result<()> {
    let profile = MachineProfile::open()?;
    let _lease = profile.lifecycle_lease()?;
    if scm::status()?.is_some_and(|status| status.dwCurrentState != SERVICE_STOPPED) {
        return Err(AppError::InvalidArgument(
            "stop the machine service before installing protected cores".into(),
        ));
    }
    let directory = profile.core_directory()?;
    directory.verify_tree()?;
    let kind = match args.core {
        InstallCore::Xray => cores::CoreKind::Xray,
        InstallCore::SingBox => cores::CoreKind::SingBox,
        InstallCore::V2Ray => cores::CoreKind::V2Ray,
    };
    let client = xrat_support::http::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(concat!("xrat/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let release = cores::fetch_release(&client, kind, args.version.as_ref(), args.prerelease)
        .await
        .map_err(AppError::InvalidArgument)?;
    let archive = cores::download_archive(&client, kind, &release, true)
        .await
        .map_err(AppError::InvalidArgument)?;
    let bytes = cores::prepare_machine_binary(kind, &release, &archive)
        .map_err(AppError::InvalidArgument)?;
    let name = format!(".{}-stage-{}.exe", kind.name(), uuid::Uuid::new_v4());
    let mut file = directory.create_file(&name)?;
    let written = file.write_all(&bytes).and_then(|()| file.sync_all());
    drop(file);
    let staged = tempfile::TempPath::try_from_path(directory.path().join(&name))?;
    written?;
    directory.verify_tree()?;
    if cores::binary_version(&staged).as_ref() != Some(&release.version) {
        return Err(AppError::InvalidArgument(
            "verified staged core did not report the requested release version".into(),
        ));
    }
    let target = directory.path().join(format!("{}.exe", kind.name()));
    staged.persist(&target).map_err(|error| {
        AppError::InvalidArgument(format!(
            "could not activate protected core; previous executable preserved: {}",
            error.error
        ))
    })?;
    directory.verify_tree()?;
    println!(
        "Installed {} v{} at {}. The machine service remains stopped.",
        kind.name(),
        release.version,
        target.display()
    );
    Ok(())
}
