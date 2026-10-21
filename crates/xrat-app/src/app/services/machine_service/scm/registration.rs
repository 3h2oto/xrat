use super::*;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows_sys::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_SERVICE_EXISTS, LocalFree};
use windows_sys::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1, SE_SERVICE,
    SetSecurityInfo,
};
use windows_sys::Win32::Security::{
    DACL_SECURITY_INFORMATION, GetSecurityDescriptorDacl, GetSecurityDescriptorOwner,
    OWNER_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION,
};

pub fn install(executable: &Path, operator_sid: &str) -> io::Result<()> {
    super::super::validate_operator(operator_sid)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?;
    let binary = command_line(executable)?;
    let manager = manager(SC_MANAGER_CREATE_SERVICE)?;
    let name = wide(SERVICE_NAME)?;
    let display = wide("XRAT machine runtime")?;
    let raw = unsafe {
        CreateServiceW(
            manager.0,
            name.as_ptr(),
            display.as_ptr(),
            SERVICE_ALL_ACCESS,
            SERVICE_WIN32_OWN_PROCESS,
            SERVICE_AUTO_START,
            SERVICE_ERROR_NORMAL,
            binary.as_ptr(),
            ptr::null(),
            ptr::null_mut(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
        )
    };
    let (service, created) = if raw.is_null() {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(ERROR_SERVICE_EXISTS as i32) {
            return Err(error);
        }
        let service = open(SERVICE_ALL_ACCESS)?;
        verify_registration(&service, &binary)?;
        (service, false)
    } else {
        (ServiceHandle(raw), true)
    };
    let result = configure(&service, operator_sid);
    if result.is_err() && created {
        // A partial installation must not leave an auto-start privileged service behind.
        if unsafe { DeleteService(service.0) } == 0 {
            return Err(io::Error::other(format!(
                "SCM configuration failed: {}; registration cleanup failed: {}",
                result
                    .err()
                    .map(|error| error.to_string())
                    .unwrap_or_default(),
                io::Error::last_os_error(),
            )));
        }
    }
    result
}

fn command_line(executable: &Path) -> io::Result<Vec<u16>> {
    let path = executable.as_os_str().encode_wide().collect::<Vec<_>>();
    if !executable.is_absolute() || path.contains(&0) || path.contains(&(b'"' as u16)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected an absolute service executable path",
        ));
    }
    let mut command = vec![b'"' as u16];
    command.extend(path);
    command.extend("\" daemon run-service\0".encode_utf16());
    Ok(command)
}

fn verify_registration(service: &ServiceHandle, expected: &[u16]) -> io::Result<()> {
    let mut needed = 0;
    unsafe { QueryServiceConfigW(service.0, ptr::null_mut(), 0, &mut needed) };
    let error = io::Error::last_os_error();
    if error.raw_os_error() != Some(ERROR_INSUFFICIENT_BUFFER as i32) || needed == 0 {
        return Err(error);
    }
    let mut buffer = vec![0usize; (needed as usize).div_ceil(size_of::<usize>())];
    let config = buffer.as_mut_ptr().cast::<QUERY_SERVICE_CONFIGW>();
    if unsafe { QueryServiceConfigW(service.0, config, needed, &mut needed) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let config = unsafe { &*config };
    let read = |value: *const u16| -> Vec<u16> {
        if value.is_null() {
            return vec![];
        }
        let mut length = 0;
        while unsafe { *value.add(length) } != 0 {
            length += 1;
        }
        unsafe { std::slice::from_raw_parts(value, length) }.to_vec()
    };
    if read(config.lpBinaryPathName) != expected[..expected.len() - 1]
        || read(config.lpServiceStartName) != "LocalSystem".encode_utf16().collect::<Vec<_>>()
        || config.dwServiceType != SERVICE_WIN32_OWN_PROCESS
        || config.dwStartType != SERVICE_AUTO_START
    {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "conflicting xrat-daemon service registration; refusing to replace it",
        ));
    }
    Ok(())
}

fn configure(service: &ServiceHandle, operator: &str) -> io::Result<()> {
    // RP/WP/LC/LO grant start, stop, query, and interrogate, never reconfiguration.
    let sddl = wide(&format!(
        "O:BAG:BAD:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;LCRPWPLO;;;{operator})"
    ))?;
    let mut descriptor = ptr::null_mut();
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let result = (|| -> io::Result<()> {
        let mut owner = ptr::null_mut();
        let mut acl = ptr::null_mut();
        let mut present = 0;
        let mut defaulted = 0;
        if unsafe { GetSecurityDescriptorOwner(descriptor, &mut owner, &mut defaulted) } == 0
            || unsafe {
                GetSecurityDescriptorDacl(descriptor, &mut present, &mut acl, &mut defaulted)
            } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if owner.is_null() || present == 0 || acl.is_null() {
            return Err(io::Error::other(
                "missing service security descriptor owner or ACL",
            ));
        }
        let status = unsafe {
            SetSecurityInfo(
                service.0,
                SE_SERVICE,
                OWNER_SECURITY_INFORMATION
                    | DACL_SECURITY_INFORMATION
                    | PROTECTED_DACL_SECURITY_INFORMATION,
                owner,
                ptr::null_mut(),
                acl,
                ptr::null(),
            )
        };
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        Ok(())
    })();
    unsafe { LocalFree(descriptor) };
    result?;
    let mut actions = [
        SC_ACTION {
            Type: SC_ACTION_RESTART,
            Delay: 5_000,
        },
        SC_ACTION {
            Type: SC_ACTION_RESTART,
            Delay: 30_000,
        },
        SC_ACTION {
            Type: SC_ACTION_NONE,
            Delay: 0,
        },
    ];
    let recovery = SERVICE_FAILURE_ACTIONSW {
        dwResetPeriod: 86_400,
        lpRebootMsg: ptr::null_mut(),
        lpCommand: ptr::null_mut(),
        cActions: actions.len() as u32,
        lpsaActions: actions.as_mut_ptr(),
    };
    if unsafe {
        ChangeServiceConfig2W(
            service.0,
            SERVICE_CONFIG_FAILURE_ACTIONS,
            (&recovery as *const SERVICE_FAILURE_ACTIONSW).cast(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let failures = SERVICE_FAILURE_ACTIONS_FLAG {
        fFailureActionsOnNonCrashFailures: 1,
    };
    if unsafe {
        ChangeServiceConfig2W(
            service.0,
            SERVICE_CONFIG_FAILURE_ACTIONS_FLAG,
            (&failures as *const SERVICE_FAILURE_ACTIONS_FLAG).cast(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub fn uninstall() -> io::Result<()> {
    let service = match open(SERVICE_STOP | SERVICE_QUERY_STATUS | 0x0001_0000) {
        Ok(service) => service,
        Err(error) if error.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST as i32) => {
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    stop_handle(&service)?;
    if unsafe { DeleteService(service.0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_command_quotes_unicode_and_spaces_and_has_no_profile_overrides() {
        let command = command_line(Path::new("C:\\Program Files\\XRAT 测试\\xrat.exe")).unwrap();
        assert_eq!(
            String::from_utf16_lossy(&command[..command.len() - 1]),
            "\"C:\\Program Files\\XRAT 测试\\xrat.exe\" daemon run-service"
        );
        assert!(command_line(Path::new("xrat.exe")).is_err());
        assert!(command_line(Path::new("C:\\bad\"path\\xrat.exe")).is_err());
    }
}
