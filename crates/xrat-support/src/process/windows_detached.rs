use super::*;
use std::os::windows::{
    ffi::OsStrExt,
    io::{AsRawHandle, FromRawHandle, OwnedHandle},
    process::ExitStatusExt,
};
use windows_sys::Win32::{
    Foundation::{DUPLICATE_SAME_ACCESS, DuplicateHandle, WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::Threading::{
        CREATE_NEW_PROCESS_GROUP, CREATE_UNICODE_ENVIRONMENT, CreateProcessW, DETACHED_PROCESS,
        DeleteProcThreadAttributeList, EXTENDED_STARTUPINFO_PRESENT, GetCurrentProcess,
        GetExitCodeProcess, INFINITE, InitializeProcThreadAttributeList,
        PROC_THREAD_ATTRIBUTE_HANDLE_LIST, PROCESS_INFORMATION, STARTF_USESTDHANDLES,
        STARTUPINFOEXW, TerminateProcess, UpdateProcThreadAttribute, WaitForSingleObject,
    },
};

struct Process {
    handle: OwnedHandle,
    id: u32,
}
impl ChildHandle for Process {
    fn id(&self) -> u32 {
        self.id
    }
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        match unsafe { WaitForSingleObject(self.handle.as_raw_handle(), 0) } {
            WAIT_OBJECT_0 => self.wait().map(Some),
            WAIT_TIMEOUT => Ok(None),
            _ => Err(io::Error::last_os_error()),
        }
    }
    fn wait(&mut self) -> io::Result<ExitStatus> {
        if unsafe { WaitForSingleObject(self.handle.as_raw_handle(), INFINITE) } != WAIT_OBJECT_0 {
            return Err(io::Error::last_os_error());
        }
        let mut status = 0;
        if unsafe { GetExitCodeProcess(self.handle.as_raw_handle(), &mut status) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(ExitStatus::from_raw(status))
    }
    fn kill(&mut self) -> io::Result<()> {
        if unsafe { TerminateProcess(self.handle.as_raw_handle(), 1) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

struct Attributes(Vec<usize>);
impl Drop for Attributes {
    fn drop(&mut self) {
        unsafe { DeleteProcThreadAttributeList(self.0.as_mut_ptr().cast()) };
    }
}

fn wide(value: &OsStr) -> io::Result<Vec<u16>> {
    let value = value.encode_wide().collect::<Vec<_>>();
    if value.contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "process string contains NUL",
        ));
    }
    Ok(value)
}

fn quoted(value: &OsStr) -> io::Result<Vec<u16>> {
    let mut result = vec![b'"' as u16];
    let mut slashes = 0;
    for unit in wide(value)? {
        if unit == b'\\' as u16 {
            slashes += 1;
            continue;
        }
        result.extend(std::iter::repeat_n(
            b'\\' as u16,
            slashes * if unit == b'"' as u16 { 2 } else { 1 },
        ));
        slashes = 0;
        if unit == b'"' as u16 {
            result.push(b'\\' as u16);
        }
        result.push(unit);
    }
    result.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2));
    result.push(b'"' as u16);
    Ok(result)
}

fn duplicate(file: &std::fs::File) -> io::Result<OwnedHandle> {
    let mut handle = std::ptr::null_mut();
    let process = unsafe { GetCurrentProcess() };
    if unsafe {
        DuplicateHandle(
            process,
            file.as_raw_handle(),
            process,
            &mut handle,
            0,
            1,
            DUPLICATE_SAME_ACCESS,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { OwnedHandle::from_raw_handle(handle) })
}

pub(super) fn spawn(spec: &CommandSpec) -> io::Result<Option<Child>> {
    if !spec.detached
        || !Path::new(&spec.program).is_absolute()
        || !Path::new(&spec.program)
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
        || !spec.env.is_empty()
        || !matches!(spec.stdin, Some(Stdio::Null))
        || !matches!(spec.stdout, Some(Stdio::Null | Stdio::File(_)))
        || !matches!(spec.stderr, Some(Stdio::Null | Stdio::File(_)))
    {
        return Ok(None);
    }
    let null = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("NUL")?;
    let input = duplicate(&null)?;
    let output = duplicate(match &spec.stdout {
        Some(Stdio::File(file)) => file,
        _ => &null,
    })?;
    let error = duplicate(match &spec.stderr {
        Some(Stdio::File(file)) => file,
        _ => &null,
    })?;
    let handles = [
        input.as_raw_handle(),
        output.as_raw_handle(),
        error.as_raw_handle(),
    ];
    let mut bytes = 0;
    unsafe { InitializeProcThreadAttributeList(std::ptr::null_mut(), 1, 0, &mut bytes) };
    if bytes == 0 {
        return Err(io::Error::last_os_error());
    }
    let mut storage = vec![0usize; bytes.div_ceil(size_of::<usize>())];
    if unsafe { InitializeProcThreadAttributeList(storage.as_mut_ptr().cast(), 1, 0, &mut bytes) }
        == 0
    {
        return Err(io::Error::last_os_error());
    }
    let mut attributes = Attributes(storage);
    if unsafe {
        UpdateProcThreadAttribute(
            attributes.0.as_mut_ptr().cast(),
            0,
            PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
            handles.as_ptr().cast(),
            size_of_val(&handles),
            std::ptr::null_mut(),
            std::ptr::null(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let mut application = wide(&spec.program)?;
    application.push(0);
    let mut line = quoted(&spec.program)?;
    for argument in &spec.args {
        line.push(b' ' as u16);
        line.extend(quoted(argument)?);
    }
    line.push(0);
    let directory = spec
        .current_dir
        .as_ref()
        .map(|path| {
            let mut value = wide(path.as_os_str())?;
            value.push(0);
            Ok::<_, io::Error>(value)
        })
        .transpose()?;
    let startup = STARTUPINFOEXW {
        StartupInfo: windows_sys::Win32::System::Threading::STARTUPINFOW {
            cb: size_of::<STARTUPINFOEXW>() as u32,
            dwFlags: STARTF_USESTDHANDLES,
            hStdInput: handles[0],
            hStdOutput: handles[1],
            hStdError: handles[2],
            ..Default::default()
        },
        lpAttributeList: attributes.0.as_mut_ptr().cast(),
    };
    let mut information = PROCESS_INFORMATION::default();
    if unsafe {
        CreateProcessW(
            application.as_ptr(),
            line.as_mut_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
            CREATE_NEW_PROCESS_GROUP
                | DETACHED_PROCESS
                | CREATE_UNICODE_ENVIRONMENT
                | EXTENDED_STARTUPINFO_PRESENT,
            std::ptr::null(),
            directory
                .as_ref()
                .map_or(std::ptr::null(), |value| value.as_ptr()),
            &startup.StartupInfo,
            &mut information,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let handle = unsafe { OwnedHandle::from_raw_handle(information.hProcess) };
    drop(unsafe { OwnedHandle::from_raw_handle(information.hThread) });
    Ok(Some(Child::from_handle(
        Box::new(Process {
            handle,
            id: information.dwProcessId,
        }),
        None,
        None,
    )))
}

#[cfg(test)]
#[path = "windows_detached_tests.rs"]
mod tests;
