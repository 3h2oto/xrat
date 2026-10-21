use std::io;
use std::os::windows::io::AsRawHandle;
use tokio::net::windows::named_pipe::{NamedPipeClient, NamedPipeServer, ServerOptions};
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
