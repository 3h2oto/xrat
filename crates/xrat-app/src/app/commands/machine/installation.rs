use sha2::{Digest, Sha256};
use std::io;
use std::path::Path;
use xrat_support::windows::ProtectedDirectory;

use crate::app::services::machine_service::{self, MachineProfile, scm};
use crate::app::{AppError, Result};
use crate::cli::{DaemonInstallArgs, DaemonUninstallArgs};

pub fn install(args: &DaemonInstallArgs, quiet: bool) -> Result<()> {
    install_inner(args, quiet).map_err(|error| installation_error("installation", error))
}

fn install_inner(args: &DaemonInstallArgs, quiet: bool) -> Result<()> {
    let operator = match &args.operator_sid {
        Some(sid) => sid.clone(),
        None => xrat_support::windows::current_user_sid()?,
    };
    machine_service::validate_operator(&operator)?;
    if args.tun {
        return Err(AppError::InvalidArgument("Windows --tun service installation is not yet available; provision capture explicitly when supported".into()));
    }
    let destination = xrat_support::windows::program_files()?.join("XRAT-Service");
    if args.dry_run {
        println!(
            "Service: {}\nIdentity: LocalSystem\nOperator: {operator}",
            scm::SERVICE_NAME
        );
        println!("Executable: {}", destination.join("xrat.exe").display());
        println!("Arguments: daemon run-service");
        println!(
            "Profile: {}",
            xrat_support::windows::program_data()?
                .join("XRAT-Service")
                .display()
        );
        println!(
            "Actions: provision protected profile and executable; register auto-start service"
        );
        if args.with_api {
            println!(
                "API login task: XRAT-Machine-API; registered operator interactive token, least privilege; starts at next logon"
            );
        }
        if args.start {
            println!("Start service after registration");
        }
        return Ok(());
    }
    let profile = MachineProfile::provision(&operator)?;
    let directory = ProtectedDirectory::create_readable(&destination, &operator)?;
    directory.verify_tree()?;
    let executable = destination.join("xrat.exe");
    let source = std::env::current_exe()?;
    match directory.create_file("xrat.exe") {
        Ok(mut output) => {
            let result = (|| -> io::Result<()> {
                io::copy(&mut std::fs::File::open(&source)?, &mut output)?;
                output.sync_all()
            })();
            drop(output);
            if let Err(error) = result {
                let _ = std::fs::remove_file(&executable);
                return Err(error.into());
            }
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            if digest(&source)? != digest(&executable)? {
                return Err(AppError::InvalidArgument("a different service executable is installed; use the coordinated service upgrade workflow instead of overwriting it".into()));
            }
        }
        Err(error) => return Err(error.into()),
    }
    directory.make_file_readable("xrat.exe")?;
    directory.verify_tree()?;
    scm::install(&executable, profile.operator_sid())?;
    if args.with_api {
        super::api_task::install(profile.operator_sid()).map_err(|error| AppError::InvalidArgument(format!("service is registered, but API login task installation failed: {error}; correct the task error and rerun installation")))?;
    }
    if args.start {
        scm::start()?;
    }
    if !quiet {
        println!("Windows machine service installed. Profile and operator registration preserved.");
        if args.with_api {
            println!(
                "API login task registered for the operator's next logon. Configure server.key and optional Web credentials in the operator's machine-client configuration. An already logged-on operator can start XRAT-Machine-API from Task Scheduler."
            );
        }
    }
    Ok(())
}

fn digest(path: &Path) -> io::Result<Vec<u8>> {
    let mut hasher = Sha256::new();
    io::copy(&mut std::fs::File::open(path)?, &mut hasher)?;
    Ok(hasher.finalize().to_vec())
}

pub fn uninstall(args: &DaemonUninstallArgs) -> Result<()> {
    if args.dry_run {
        println!(
            "Would remove the owned API login task and stop/remove SCM service {}. Protected profile, executable, and user UI/auth data preserved.",
            scm::SERVICE_NAME
        );
        return Ok(());
    }
    super::api_task::uninstall()
        .map_err(|error| installation_error("API login task removal", error))?;
    scm::uninstall().map_err(|error| installation_error("removal", error.into()))?;
    println!("Windows machine service removed. Protected profile and executable preserved.");
    Ok(())
}

fn installation_error(operation: &str, error: AppError) -> AppError {
    match error {
        AppError::Io(error) => {
            let hint = if error.kind() == io::ErrorKind::PermissionDenied {
                "; rerun from an elevated administrator shell. Use --operator-sid when installing for another user"
            } else {
                ""
            };
            AppError::InvalidArgument(format!("Windows service {operation} failed: {error}{hint}"))
        }
        error => error,
    }
}
