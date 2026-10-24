use crate::app::services::{
    machine_service::{MachineProfile, scm},
    windows_tun,
};
use crate::app::{AppError, Result};
use crate::cli::TunSetupArgs;
use std::io::Write;
use windows_sys::Win32::System::Services::SERVICE_STOPPED;
use xrat_support::windows::ProtectedDirectory;

pub(super) async fn run(args: &TunSetupArgs) -> Result<()> {
    if args.dry_run {
        println!(
            "Provision pinned Wintun 0.14.1 from {}",
            windows_tun::ARCHIVE_URL
        );
        println!(
            "Verify archive/DLL/license SHA-256 and Windows Authenticode; preserve license; activate beside protected cores while service is stopped."
        );
        println!(
            "Destination: {}",
            xrat_support::windows::program_data()?
                .join("XRAT-Service/cores/wintun.dll")
                .display()
        );
        println!(
            "This prepares files only. Start the machine service, then use ordinary --profile machine tun enable."
        );
        return Ok(());
    }
    if !xrat_support::windows::is_current_process_elevated()? {
        return Err(AppError::InvalidArgument("machine TUN setup requires an elevated administrator shell; routine TUN control uses the ordinary operator account".into()));
    }
    let profile = MachineProfile::open()?;
    let _lease = profile.lifecycle_lease()?;
    if scm::status()?.is_some_and(|status| status.dwCurrentState != SERVICE_STOPPED) {
        return Err(AppError::InvalidArgument(
            "stop the machine service before provisioning TUN files".into(),
        ));
    }
    let directory = profile.core_directory()?;
    directory.verify_tree()?;
    if windows_tun::verify_installed(directory.path()).is_ok() {
        windows_tun::verify_signature(&directory.path().join("wintun.dll"))?;
        println!(
            "Managed Wintun 0.14.1 and license are already verified. Service remains stopped."
        );
        return Ok(());
    }
    let client = xrat_support::http::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .redirect(xrat_support::http::RedirectPolicy::None)
        .user_agent(concat!("xrat/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let mut response = client.get(windows_tun::ARCHIVE_URL).send().await?;
    if !response.status().is_success() {
        return Err(AppError::InvalidArgument(format!(
            "Wintun download returned HTTP {}",
            response.status()
        )));
    }
    if response
        .content_length()
        .is_some_and(|size| size > windows_tun::ARCHIVE_LIMIT as u64)
    {
        return Err(AppError::InvalidArgument(
            "Wintun archive exceeds 4 MiB".into(),
        ));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if chunk.len() > windows_tun::ARCHIVE_LIMIT.saturating_sub(bytes.len()) {
            return Err(AppError::InvalidArgument(
                "Wintun archive exceeds 4 MiB".into(),
            ));
        }
        bytes.extend(chunk);
    }
    let package =
        windows_tun::prepare(&bytes, std::env::consts::ARCH).map_err(AppError::InvalidArgument)?;
    let dll = stage(&directory, "dll", &package.dll)?;
    let license = stage(&directory, "txt", &package.license)?;
    directory.verify_tree()?;
    windows_tun::verify_signature(&dll)?;
    activate(license, &directory.path().join("wintun-LICENSE.txt"))?;
    activate(dll, &directory.path().join("wintun.dll"))?;
    directory.verify_tree()?;
    windows_tun::verify_installed(directory.path()).map_err(AppError::InvalidArgument)?;
    println!(
        "Prepared signed Wintun 0.14.1 and its license beside protected cores. Service remains stopped; start it before enabling machine capture."
    );
    Ok(())
}

fn stage(
    directory: &ProtectedDirectory,
    extension: &str,
    bytes: &[u8],
) -> Result<tempfile::TempPath> {
    let name = format!(".wintun-{}.{}", uuid::Uuid::new_v4(), extension);
    let mut file = directory.create_file(&name)?;
    let result = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    let staged = tempfile::TempPath::try_from_path(directory.path().join(name))?;
    result?;
    Ok(staged)
}

fn activate(staged: tempfile::TempPath, target: &std::path::Path) -> Result<()> {
    staged.persist(target).map_err(|error| {
        AppError::InvalidArgument(format!(
            "could not activate {}: {}; existing file preserved",
            target.display(),
            error.error
        ))
    })?;
    Ok(())
}
