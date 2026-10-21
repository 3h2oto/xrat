use std::io;
use std::ptr;
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{
    ERROR_SERVICE_ALREADY_RUNNING, ERROR_SERVICE_DOES_NOT_EXIST, ERROR_SERVICE_NOT_ACTIVE,
};
use windows_sys::Win32::System::Services::*;

mod registration;
pub use registration::{install, uninstall};

pub const SERVICE_NAME: &str = "xrat-daemon";

pub(super) fn wide(value: &str) -> io::Result<Vec<u16>> {
    if value.contains('\0') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "NUL in SCM argument",
        ));
    }
    Ok(value.encode_utf16().chain(Some(0)).collect())
}

pub(super) struct ServiceHandle(pub SC_HANDLE);

impl ServiceHandle {
    pub fn from_raw(handle: SC_HANDLE) -> io::Result<Self> {
        if handle.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self(handle))
        }
    }
}

impl Drop for ServiceHandle {
    fn drop(&mut self) {
        unsafe { CloseServiceHandle(self.0) };
    }
}

pub(super) fn manager(access: u32) -> io::Result<ServiceHandle> {
    ServiceHandle::from_raw(unsafe { OpenSCManagerW(ptr::null(), ptr::null(), access) })
}

pub(super) fn open(access: u32) -> io::Result<ServiceHandle> {
    let manager = manager(SC_MANAGER_CONNECT)?;
    let name = wide(SERVICE_NAME)?;
    ServiceHandle::from_raw(unsafe { OpenServiceW(manager.0, name.as_ptr(), access) })
}

pub(super) fn query(service: &ServiceHandle) -> io::Result<SERVICE_STATUS_PROCESS> {
    let mut status = SERVICE_STATUS_PROCESS::default();
    let mut needed = 0;
    if unsafe {
        QueryServiceStatusEx(
            service.0,
            SC_STATUS_PROCESS_INFO,
            (&mut status as *mut SERVICE_STATUS_PROCESS).cast(),
            size_of::<SERVICE_STATUS_PROCESS>() as u32,
            &mut needed,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(status)
}

pub fn status() -> io::Result<Option<SERVICE_STATUS_PROCESS>> {
    match open(SERVICE_QUERY_STATUS) {
        Ok(service) => query(&service).map(Some),
        Err(error) if error.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST as i32) => Ok(None),
        Err(error) => Err(error),
    }
}

pub(super) fn wait(service: &ServiceHandle, expected: u32) -> io::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let status = query(service)?;
        if status.dwCurrentState == expected {
            return Ok(());
        }
        if expected == SERVICE_RUNNING && status.dwCurrentState == SERVICE_STOPPED {
            return Err(io::Error::other(format!(
                "XRAT service stopped during startup (Windows error {}, service error {})",
                status.dwWin32ExitCode, status.dwServiceSpecificExitCode,
            )));
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "XRAT SCM transition timed out",
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub fn start() -> io::Result<()> {
    let service = open(SERVICE_START | SERVICE_QUERY_STATUS)?;
    let current = query(&service)?.dwCurrentState;
    if current == SERVICE_RUNNING {
        return Ok(());
    }
    if current == SERVICE_START_PENDING {
        return wait(&service, SERVICE_RUNNING);
    }
    if current == SERVICE_STOP_PENDING {
        wait(&service, SERVICE_STOPPED)?;
    }
    if unsafe { StartServiceW(service.0, 0, ptr::null()) } == 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(ERROR_SERVICE_ALREADY_RUNNING as i32) {
            return Err(error);
        }
    }
    wait(&service, SERVICE_RUNNING)
}

pub(super) fn stop_handle(service: &ServiceHandle) -> io::Result<()> {
    let current = query(service)?.dwCurrentState;
    if current == SERVICE_STOPPED {
        return Ok(());
    }
    if current == SERVICE_STOP_PENDING {
        return wait(service, SERVICE_STOPPED);
    }
    if current == SERVICE_START_PENDING {
        // SCM does not accept STOP during START_PENDING. A failed start is already stopped.
        if let Err(error) = wait(service, SERVICE_RUNNING)
            && query(service)?.dwCurrentState != SERVICE_STOPPED
        {
            return Err(error);
        }
        if query(service)?.dwCurrentState == SERVICE_STOPPED {
            return Ok(());
        }
    }
    let mut status = SERVICE_STATUS::default();
    if unsafe { ControlService(service.0, SERVICE_CONTROL_STOP, &mut status) } == 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(ERROR_SERVICE_NOT_ACTIVE as i32) {
            return Err(error);
        }
    }
    wait(service, SERVICE_STOPPED)
}

pub fn stop() -> io::Result<()> {
    match open(SERVICE_STOP | SERVICE_QUERY_STATUS) {
        Ok(service) => stop_handle(&service),
        Err(error) if error.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST as i32) => Ok(()),
        Err(error) => Err(error),
    }
}

pub fn restart() -> io::Result<()> {
    stop()?;
    start()
}
