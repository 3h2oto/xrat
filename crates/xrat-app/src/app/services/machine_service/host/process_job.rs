use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::sync::OnceLock;
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};
use windows_sys::Win32::System::Threading::GetCurrentProcess;

// The non-inheritable handle must outlive SCM callbacks and close only at process
// exit. A scope guard would terminate the service itself when it is dropped.
static SERVICE_JOB: OnceLock<Result<OwnedHandle, i32>> = OnceLock::new();

pub(super) fn contain_service_process() -> io::Result<()> {
    SERVICE_JOB
        .get_or_init(|| create().map_err(|error| error.raw_os_error().unwrap_or(1)))
        .as_ref()
        .map(|_| ())
        .map_err(|code| io::Error::from_raw_os_error(*code))
}

fn create() -> io::Result<OwnedHandle> {
    let raw = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
    if raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    let job = unsafe { OwnedHandle::from_raw_handle(raw) };
    let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    if unsafe {
        SetInformationJobObject(
            job.as_raw_handle(),
            JobObjectExtendedLimitInformation,
            (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of_val(&limits) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if unsafe { AssignProcessToJobObject(job.as_raw_handle(), GetCurrentProcess()) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(job)
}

#[cfg(test)]
mod tests;
