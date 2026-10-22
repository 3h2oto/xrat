use crate::app::{AppError, Result};
use std::{
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::Path,
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT},
    Security::*,
    System::Threading::*,
};
fn quoted(path: &Path) -> Result<Vec<u16>> {
    let mut output = vec![34];
    let mut slashes = 0;
    for value in path.as_os_str().encode_wide() {
        if value == 0 {
            return Err(AppError::InvalidArgument("NUL in updater path".into()));
        }
        if value == 92 {
            slashes += 1;
            continue;
        }
        output.extend(std::iter::repeat_n(
            92,
            if value == 34 {
                slashes * 2 + 1
            } else {
                slashes
            },
        ));
        slashes = 0;
        output.push(value);
    }
    output.extend(std::iter::repeat_n(92, slashes * 2));
    output.push(34);
    Ok(output)
}
fn start(manifest: &Path) -> Result<OwnedHandle> {
    let mut raw = std::ptr::null_mut();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw) } == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let current = unsafe { OwnedHandle::from_raw_handle(raw) };
    let mut linked = TOKEN_LINKED_TOKEN::default();
    let mut returned = 0;
    if unsafe {
        GetTokenInformation(
            current.as_raw_handle(),
            TokenLinkedToken,
            (&mut linked as *mut TOKEN_LINKED_TOKEN).cast(),
            size_of::<TOKEN_LINKED_TOKEN>() as u32,
            &mut returned,
        )
    } == 0
    {
        return Err(AppError::InvalidArgument("user updater needs the ordinary linked user token; rerun the upgrade from a non-elevated shell".into()));
    }
    if returned as usize != size_of::<TOKEN_LINKED_TOKEN>() || linked.LinkedToken.is_null() {
        return Err(AppError::InvalidArgument(
            "invalid ordinary linked user token".into(),
        ));
    }
    let linked = unsafe { OwnedHandle::from_raw_handle(linked.LinkedToken) };
    let mut elevation = TOKEN_ELEVATION::default();
    if unsafe {
        GetTokenInformation(
            linked.as_raw_handle(),
            TokenElevation,
            (&mut elevation as *mut TOKEN_ELEVATION).cast(),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        )
    } == 0
        || returned as usize != size_of::<TOKEN_ELEVATION>()
        || elevation.TokenIsElevated != 0
    {
        return Err(AppError::InvalidArgument(
            "refusing to apply a user upgrade with an elevated token".into(),
        ));
    }
    let executable = std::env::current_exe()?;
    let application: Vec<u16> = executable
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let mut line = quoted(&executable)?;
    line.extend(" upgrade --apply-transaction ".encode_utf16());
    line.extend(quoted(manifest)?);
    line.push(0);
    let startup = STARTUPINFOW {
        cb: size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut info = PROCESS_INFORMATION::default();
    if unsafe {
        CreateProcessWithTokenW(
            linked.as_raw_handle(),
            LOGON_WITH_PROFILE,
            application.as_ptr(),
            line.as_mut_ptr(),
            CREATE_NO_WINDOW,
            std::ptr::null(),
            std::ptr::null(),
            &startup,
            &mut info,
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error().into());
    }
    let process = unsafe { OwnedHandle::from_raw_handle(info.hProcess) };
    drop(unsafe { OwnedHandle::from_raw_handle(info.hThread) });
    Ok(process)
}
pub(super) async fn apply(manifest: &Path) -> Result<()> {
    let process = start(manifest)?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(240);
    loop {
        match unsafe { WaitForSingleObject(process.as_raw_handle(), 0) } {
            WAIT_OBJECT_0 => {
                let mut code = 1;
                if unsafe { GetExitCodeProcess(process.as_raw_handle(), &mut code) } == 0 {
                    return Err(std::io::Error::last_os_error().into());
                }
                if code != 0 {
                    return Err(AppError::InvalidArgument(format!(
                        "ordinary user updater failed ({code}); inspect upgrade --status"
                    )));
                }
                return Ok(());
            }
            WAIT_TIMEOUT => {}
            _ => return Err(std::io::Error::last_os_error().into()),
        }
        if tokio::time::Instant::now() >= deadline {
            unsafe { TerminateProcess(process.as_raw_handle(), 1) };
            return Err(AppError::InvalidArgument(
                "ordinary updater timed out; inspect its retained transaction".into(),
            ));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn quotes_unicode_spaces_quotes_and_trailing_slashes() {
        for (input, expected) in [
            (r"C:\upgrade م\xrat.exe", "\"C:\\upgrade م\\xrat.exe\""),
            ("C:\\tail\\", "\"C:\\tail\\\\\""),
            ("a\"b", "\"a\\\"b\""),
        ] {
            assert_eq!(
                String::from_utf16(&super::quoted(std::path::Path::new(input)).unwrap()).unwrap(),
                expected
            );
        }
    }
}
