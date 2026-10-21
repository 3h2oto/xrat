use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::ptr;

use windows_sys::Win32::Foundation::{HANDLE, LocalFree};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
};
use windows_sys::Win32::Security::{
    GetTokenInformation, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER, TokenUser,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};

mod protected_directory;
pub use protected_directory::{ProtectedDirectory, program_data, program_files};

pub fn current_user_sid() -> io::Result<String> {
    unsafe { user_sid(GetCurrentProcess()) }
}

pub fn process_user_sid(pid: u32) -> io::Result<String> {
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process.is_null() {
        return Err(io::Error::last_os_error());
    }
    let process = unsafe { OwnedHandle::from_raw_handle(process) };
    unsafe { user_sid(process.as_raw_handle()) }
}

unsafe fn user_sid(process: HANDLE) -> io::Result<String> {
    let mut token = ptr::null_mut();
    if unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let token = unsafe { OwnedHandle::from_raw_handle(token) };
    token_user_sid(token.as_raw_handle())
}

fn token_user_sid(token: HANDLE) -> io::Result<String> {
    let mut required = 0;
    unsafe { GetTokenInformation(token, TokenUser, ptr::null_mut(), 0, &mut required) };
    if required == 0 || required > 1024 * 1024 {
        return Err(io::Error::last_os_error());
    }
    let mut buffer = vec![0usize; (required as usize).div_ceil(size_of::<usize>())];
    if unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            required,
            &mut required,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let user = unsafe { &*buffer.as_ptr().cast::<TOKEN_USER>() };
    sid_string(user.User.Sid)
}

fn sid_string(value: windows_sys::Win32::Security::PSID) -> io::Result<String> {
    let mut sid = ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(value, &mut sid) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let mut length = 0;
    while unsafe { *sid.add(length) } != 0 {
        length += 1;
    }
    let result = String::from_utf16(unsafe { std::slice::from_raw_parts(sid, length) })
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
    unsafe { LocalFree(sid.cast()) };
    result
}

pub struct SecurityDescriptor(*mut std::ffi::c_void);

impl SecurityDescriptor {
    pub fn for_owner(owner_sid: &str) -> io::Result<Self> {
        if !owner_sid.starts_with("S-1-")
            || !owner_sid
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'S' | b'-'))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid owner SID",
            ));
        }
        // Clients get read/write-data rights, without FILE_CREATE_PIPE_INSTANCE.
        let sddl =
            format!("D:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;0x0012019b;;;{owner_sid})S:(ML;;NW;;;ME)");
        Self::from_sddl(&sddl)
    }

    pub fn for_machine_operator(owner_sid: &str) -> io::Result<Self> {
        Self::for_owner(owner_sid)?;
        Self::from_sddl(&format!(
            "O:SYG:SYD:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;0x0012019b;;;{owner_sid})S:(ML;;NW;;;ME)"
        ))
    }

    fn from_sddl(sddl: &str) -> io::Result<Self> {
        let wide = sddl.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        let mut descriptor = ptr::null_mut();
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wide.as_ptr(),
                1,
                &mut descriptor,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(descriptor))
    }

    pub fn attributes(&self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0,
            bInheritHandle: 0,
        }
    }
}

pub fn require_system_owner(handle: &impl AsRawHandle) -> io::Result<()> {
    use windows_sys::Win32::Security::Authorization::{GetSecurityInfo, SE_FILE_OBJECT};
    use windows_sys::Win32::Security::OWNER_SECURITY_INFORMATION;
    let mut owner = ptr::null_mut();
    let mut descriptor = ptr::null_mut();
    let result = unsafe {
        GetSecurityInfo(
            handle.as_raw_handle(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION,
            &mut owner,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut descriptor,
        )
    };
    if result != 0 {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    let _descriptor = SecurityDescriptor(descriptor);
    if owner.is_null() || sid_string(owner)? != "S-1-5-18" {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "machine gateway pipe is not owned by SYSTEM",
        ));
    }
    Ok(())
}

pub fn named_pipe_client_sid(pipe: &impl AsRawHandle) -> io::Result<String> {
    use windows_sys::Win32::Security::RevertToSelf;
    use windows_sys::Win32::System::Pipes::ImpersonateNamedPipeClient;
    use windows_sys::Win32::System::Threading::{GetCurrentThread, OpenThreadToken};

    struct Revert;
    impl Drop for Revert {
        fn drop(&mut self) {
            if unsafe { RevertToSelf() } == 0 {
                std::process::abort();
            }
        }
    }

    if unsafe { ImpersonateNamedPipeClient(pipe.as_raw_handle()) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let _revert = Revert;
    let mut token = ptr::null_mut();
    if unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let token = unsafe { OwnedHandle::from_raw_handle(token) };
    token_user_sid(token.as_raw_handle())
}

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        unsafe { LocalFree(self.0) };
    }
}

pub fn process_creation_time(pid: u32) -> io::Result<u64> {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return Err(io::Error::last_os_error());
    }
    let handle = unsafe { OwnedHandle::from_raw_handle(handle) };
    creation_time(handle.as_raw_handle())
}

fn creation_time(handle: HANDLE) -> io::Result<u64> {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Threading::GetProcessTimes;
    let mut creation = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    if unsafe { GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(u64::from(creation.dwHighDateTime) << 32 | u64::from(creation.dwLowDateTime))
}

pub fn wait_process_exit(
    pid: u32,
    expected_creation: u64,
    timeout: std::time::Duration,
) -> io::Result<()> {
    use windows_sys::Win32::Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT};
    use windows_sys::Win32::System::Threading::WaitForSingleObject;
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | 0x0010_0000, 0, pid) };
    if handle.is_null() {
        let error = io::Error::last_os_error();
        return if error.raw_os_error() == Some(87) {
            Ok(())
        } else {
            Err(error)
        };
    }
    let handle = unsafe { OwnedHandle::from_raw_handle(handle) };
    if creation_time(handle.as_raw_handle())? != expected_creation {
        return Ok(());
    }
    match unsafe {
        WaitForSingleObject(
            handle.as_raw_handle(),
            timeout.as_millis().min(u128::from(u32::MAX - 1)) as u32,
        )
    } {
        WAIT_OBJECT_0 => Ok(()),
        WAIT_TIMEOUT => Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "XRAT is still running; exit the upgrading client and retry",
        )),
        _ => Err(io::Error::last_os_error()),
    }
}
