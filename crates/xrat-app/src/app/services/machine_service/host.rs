use std::io;
use std::ptr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use windows_sys::Win32::Foundation::{ERROR_CALL_NOT_IMPLEMENTED, ERROR_SERVICE_SPECIFIC_ERROR};
use windows_sys::Win32::System::Services::*;

use super::{MachineProfile, SYSTEM_SID, scm};
use crate::app::daemon::{ipc, supervisor};
use crate::app::runtime_service::RuntimeService;
use crate::app::{AppError, Result, events};
use xrat_support::cancel::{CancellationFlag, CancellationReceiver};

mod diagnostics;

static STOP: AtomicBool = AtomicBool::new(false);
static REPORTER: Mutex<Reporter> = Mutex::new(Reporter {
    handle: 0,
    state: SERVICE_STOPPED,
    checkpoint: 0,
});

struct Reporter {
    handle: usize,
    state: u32,
    checkpoint: u32,
}

fn report(state: u32, failed: bool) -> io::Result<()> {
    let mut reporter = REPORTER
        .lock()
        .map_err(|_| io::Error::other("SCM status lock poisoned"))?;
    if STOP.load(Ordering::SeqCst) && state == SERVICE_RUNNING {
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "service stop requested during startup",
        ));
    }
    reporter.checkpoint = if matches!(state, SERVICE_START_PENDING | SERVICE_STOP_PENDING) {
        if reporter.state == state {
            reporter.checkpoint.saturating_add(1)
        } else {
            1
        }
    } else {
        0
    };
    reporter.state = state;
    let status = SERVICE_STATUS {
        dwServiceType: SERVICE_WIN32_OWN_PROCESS,
        dwCurrentState: state,
        dwControlsAccepted: if state == SERVICE_RUNNING {
            SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN
        } else {
            0
        },
        dwWin32ExitCode: if failed {
            ERROR_SERVICE_SPECIFIC_ERROR
        } else {
            0
        },
        dwServiceSpecificExitCode: u32::from(failed),
        dwCheckPoint: reporter.checkpoint,
        dwWaitHint: if reporter.checkpoint > 0 { 10_000 } else { 0 },
    };
    if unsafe { SetServiceStatus(reporter.handle as SERVICE_STATUS_HANDLE, &status) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

unsafe extern "system" fn control(
    code: u32,
    _: u32,
    _: *mut std::ffi::c_void,
    _: *mut std::ffi::c_void,
) -> u32 {
    match code {
        SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
            STOP.store(true, Ordering::SeqCst);
            match report(SERVICE_STOP_PENDING, false) {
                Ok(()) => 0,
                Err(error) => error.raw_os_error().unwrap_or(1) as u32,
            }
        }
        SERVICE_CONTROL_INTERROGATE => 0,
        _ => ERROR_CALL_NOT_IMPLEMENTED,
    }
}

pub fn run_service() -> Result<()> {
    if xrat_support::windows::current_user_sid()? != SYSTEM_SID {
        return Err(AppError::InvalidArgument(
            "Windows service entrypoint requires LocalSystem and SCM".into(),
        ));
    }
    let mut name = scm::wide(scm::SERVICE_NAME)?;
    let entries = [
        SERVICE_TABLE_ENTRYW {
            lpServiceName: name.as_mut_ptr(),
            lpServiceProc: Some(service_main),
        },
        SERVICE_TABLE_ENTRYW {
            lpServiceName: ptr::null_mut(),
            lpServiceProc: None,
        },
    ];
    if unsafe { StartServiceCtrlDispatcherW(entries.as_ptr()) } == 0 {
        return Err(io::Error::last_os_error().into());
    }
    Ok(())
}

unsafe extern "system" fn service_main(_: u32, _: *mut *mut u16) {
    let result = std::panic::catch_unwind(|| -> Result<()> {
        let name = scm::wide(scm::SERVICE_NAME)?;
        let handle =
            unsafe { RegisterServiceCtrlHandlerExW(name.as_ptr(), Some(control), ptr::null()) };
        if handle.is_null() {
            return Err(io::Error::last_os_error().into());
        }
        REPORTER
            .lock()
            .map_err(|_| io::Error::other("SCM status lock poisoned"))?
            .handle = handle as usize;
        report(SERVICE_START_PENDING, false)?;
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(host())
    });
    let failed = !matches!(result, Ok(Ok(())));
    if let Ok(Err(error)) = &result {
        let message = diagnostics::describe(error);
        tracing::error!(%message, "Windows machine service failed");
        diagnostics::failure(&message);
    } else if result.is_err() {
        diagnostics::failure("Windows machine service panicked");
    }
    let _ = report(SERVICE_STOPPED, failed);
}

async fn pending<T>(future: impl std::future::Future<Output = Result<T>>, state: u32) -> Result<T> {
    tokio::pin!(future);
    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    let deadline = tokio::time::sleep(Duration::from_secs(35));
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            result = &mut future => return result,
            _ = ticker.tick() => report(state, false)?,
            _ = &mut deadline => return Err(io::Error::new(io::ErrorKind::TimedOut, "machine service lifecycle operation timed out").into()),
        }
    }
}

async fn host() -> Result<()> {
    let profile = MachineProfile::open()?;
    let _lease = profile.lifecycle_lease()?;
    let mut context = pending(profile.context(), SERVICE_START_PENDING).await?;
    pending(
        super::lifecycle::prepare_runtime(&mut context),
        SERVICE_START_PENDING,
    )
    .await?;
    let (sender, receiver) = supervisor::channel(32);
    let cancel = CancellationFlag::new();
    let mut tasks = tokio::task::JoinSet::new();
    let supervisor_context = context.clone();
    tasks.spawn(async move {
        supervisor::run_machine(receiver, supervisor_context).await;
        Ok(())
    });
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    tasks.spawn(super::transport::serve_with_ready(
        profile,
        context.clone(),
        sender.clone(),
        CancellationReceiver::from_flag(&cancel),
        Some(ready_tx),
    ));
    let operation = async {
        pending(async {
            ready_rx.await.map_err(|error| io::Error::other(error.to_string()))?;
            Ok(())
        }, SERVICE_START_PENDING).await?;
        report(SERVICE_RUNNING, false)?;
        events::record(
            &context.db, events::LEVEL_INFO, events::SOURCE_DAEMON,
            "machine_service_started", "Windows machine service started; waiting for explicit connect",
            None, None, None,
        ).await;
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    if STOP.load(Ordering::SeqCst) { return Ok(()); }
                }
                result = tasks.join_next() => return Err(AppError::InvalidArgument(format!("machine service task exited unexpectedly: {result:?}"))),
            }
        }
    }.await;
    let status = report(SERVICE_STOP_PENDING, false);
    cancel.cancel();
    let shutdown = pending(
        async {
            let response = ipc::transport::daemon_shutdown_response_via_supervisor(sender).await?;
            if !response.ok {
                return Err(AppError::InvalidArgument(response.message));
            }
            Ok(())
        },
        SERVICE_STOP_PENDING,
    )
    .await;
    tasks.abort_all();
    while tasks.join_next().await.is_some() {}
    let cleanup = pending(
        async {
            let runtime = RuntimeService::new(&context);
            let _ = runtime
                .reconcile_reattach_on_daemon_start(&uuid::Uuid::new_v4().to_string())
                .await?;
            runtime.disconnect().await?;
            runtime.cleanup_inactive_tun_ownership().await?;
            Ok(())
        },
        SERVICE_STOP_PENDING,
    )
    .await;
    let errors = [
        operation.err().map(|error| diagnostics::describe(&error)),
        status.err().map(|error| error.to_string()),
        shutdown.err().map(|error| diagnostics::describe(&error)),
        cleanup.err().map(|error| diagnostics::describe(&error)),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if !errors.is_empty() {
        return Err(io::Error::other(errors.join("; ")).into());
    }
    events::record(
        &context.db,
        events::LEVEL_INFO,
        events::SOURCE_DAEMON,
        "machine_service_stopped",
        "Windows machine service stopped",
        None,
        None,
        None,
    )
    .await;
    Ok(())
}
