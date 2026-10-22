use super::policy;
use crate::app::{AppError, Result, context::AppContext};
use xrat_support::{
    net::{self},
    readiness::RuntimeProcessPorts,
};

pub(crate) fn check_engine(context: &AppContext, ports: &RuntimeProcessPorts) -> Result<()> {
    if xrat_support::windows::current_user_sid()? != "S-1-5-18" {
        return Err(AppError::InvalidArgument("Windows TUN requires the protected LocalSystem runtime; use --profile machine after elevated setup".into()));
    }
    let profile = crate::app::services::machine_service::MachineProfile::open()?;
    if context.runtime_paths.config_path != profile.config_path() {
        return Err(AppError::InvalidArgument(
            "Windows TUN requires the protected machine configuration".into(),
        ));
    }
    let directory = profile.core_directory()?;
    directory.verify_tree()?;
    let (binary, minimum, maximum) = match context.app_config.runtime.engine.as_str() {
        "xray" => {
            super::verify_installed(directory.path()).map_err(AppError::InvalidArgument)?;
            (
                &context.runtime_paths.xray_path,
                semver::Version::new(26, 9, 30),
                semver::Version::new(27, 0, 0),
            )
        }
        "sing-box" => (
            &context.runtime_paths.sing_box_path,
            semver::Version::new(1, 13, 21),
            semver::Version::new(1, 14, 0),
        ),
        _ => {
            return Err(AppError::InvalidArgument(
                "Windows managed TUN supports Xray and sing-box; V2Ray TUN is unavailable".into(),
            ));
        }
    };
    let expected = directory
        .path()
        .join(format!("{}.exe", context.app_config.runtime.engine));
    if binary != &expected {
        return Err(AppError::InvalidArgument(
            "Windows TUN requires a protected managed engine".into(),
        ));
    }
    let version =
        crate::app::services::cores::binary_version_with_spawner(binary, ports.spawner.clone())
            .ok_or_else(|| {
                AppError::InvalidArgument(
                    "could not determine the protected TUN engine version".into(),
                )
            })?;
    if version < minimum || version >= maximum {
        return Err(AppError::InvalidArgument(format!(
            "Windows TUN requires {} >= {minimum} and < {maximum}; installed {version}",
            context.app_config.runtime.engine
        )));
    }
    policy::dns_addresses(&context.app_config.runtime.tun.address)
        .map_err(AppError::InvalidArgument)?;
    Ok(())
}
pub(crate) fn preflight(context: &AppContext) -> Result<()> {
    let record = crate::app::runtime_service::tun_ownership::load_ownership(
        &context.runtime_paths.runtime_dir,
    );
    let owned = record
        .as_ref()
        .and_then(|record| record.interface_identity.as_deref());
    policy::preflight(
        &net::windows_network_snapshot()?,
        &context.app_config.runtime.tun,
        owned,
    )
    .map_err(AppError::InvalidArgument)
}
pub(crate) fn ready(context: &AppContext, identity: &str) -> Result<bool> {
    policy::ready(
        &net::windows_network_snapshot()?,
        &context.app_config.runtime.tun,
        identity,
    )
    .map_err(AppError::InvalidArgument)
}
pub(crate) fn verify_release(identity: &str) -> Result<()> {
    let (_, luid) = identity.rsplit_once('@').ok_or_else(|| {
        AppError::InvalidArgument(
            "Windows TUN journal lacks its verified LUID; ownership retained for manual recovery"
                .into(),
        )
    })?;
    let luid = u64::from_str_radix(luid, 16)
        .map_err(|_| AppError::InvalidArgument("invalid Windows TUN ownership identity".into()))?;
    let snapshot = net::windows_network_snapshot()?;
    if snapshot.adapters.iter().any(|adapter| adapter.luid == luid)
        || snapshot.routes.iter().any(|route| route.luid == luid)
    {
        return Err(AppError::InvalidArgument("Windows adapter or owned routes remain after engine exit; ownership retained for retry".into()));
    }
    Ok(())
}
