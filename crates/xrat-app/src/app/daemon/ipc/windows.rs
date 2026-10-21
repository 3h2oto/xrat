use std::io;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::AsRawHandle;
use tokio::net::windows::named_pipe::{NamedPipeClient, NamedPipeServer, ServerOptions};
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAG_OVERLAPPED, FILE_GENERIC_READ, FILE_WRITE_DATA, OPEN_EXISTING,
    SECURITY_IDENTIFICATION, SECURITY_SQOS_PRESENT,
};
use windows_sys::Win32::System::Pipes::{GetNamedPipeClientProcessId, GetNamedPipeServerProcessId};

pub(super) fn listener(endpoint: &std::path::Path, first: bool) -> io::Result<NamedPipeServer> {
    let owner = xrat_support::windows::current_user_sid()?;
    let descriptor = xrat_support::windows::SecurityDescriptor::for_owner(&owner)?;
    let attributes = descriptor.attributes();
    unsafe {
        ServerOptions::new()
            .first_pipe_instance(first)
            .reject_remote_clients(true)
            .create_with_security_attributes_raw(
                endpoint,
                (&attributes as *const windows_sys::Win32::Security::SECURITY_ATTRIBUTES)
                    .cast_mut()
                    .cast(),
            )
    }
}

pub(super) fn connect(endpoint: &std::path::Path) -> io::Result<NamedPipeClient> {
    let name = endpoint
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    // GENERIC_WRITE also requests FILE_CREATE_PIPE_INSTANCE, which clients must not have.
    let handle = unsafe {
        CreateFileW(
            name.as_ptr(),
            FILE_GENERIC_READ | FILE_WRITE_DATA,
            0,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED | SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    unsafe { NamedPipeClient::from_raw_handle(handle) }
}

pub(super) fn verify_client(pipe: &NamedPipeServer) -> io::Result<()> {
    let mut pid = 0;
    if unsafe { GetNamedPipeClientProcessId(pipe.as_raw_handle(), &mut pid) } == 0 {
        return Err(io::Error::last_os_error());
    }
    verify_user(pid)
}

pub(super) fn verify_server(pipe: &NamedPipeClient) -> io::Result<()> {
    let mut pid = 0;
    if unsafe { GetNamedPipeServerProcessId(pipe.as_raw_handle(), &mut pid) } == 0 {
        return Err(io::Error::last_os_error());
    }
    verify_user(pid)
}

fn verify_user(pid: u32) -> io::Result<()> {
    if xrat_support::windows::process_user_sid(pid)? != xrat_support::windows::current_user_sid()? {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "daemon peer belongs to a different user",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "windows_security_tests.rs"]
mod tests;
