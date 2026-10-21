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
    if args.with_api || args.tun {
        return Err(AppError::InvalidArgument("Windows --with-api and --tun integration is not yet available; install the service without these flags".into()));
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
        if args.start {
            println!("Start service after registration");
        }
        return Ok(());
    }
    let profile = MachineProfile::provision(&operator)?;
    let directory = ProtectedDirectory::create(&destination)?;
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
    directory.verify_tree()?;
    scm::install(&executable, profile.operator_sid())?;
    if args.start {
        scm::start()?;
    }
    if !quiet {
        println!("Windows machine service installed. Profile and operator registration preserved.");
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
            "Would stop and remove SCM service {}. Protected profile and executable preserved.",
            scm::SERVICE_NAME
        );
        return Ok(());
    }
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
