use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::ptr;
use windows_sys::Win32::Security::{
    CreateRestrictedToken, CreateWellKnownSid, DISABLE_MAX_PRIVILEGE, EqualSid,
    GetTokenInformation, SECURITY_MAX_SID_SIZE, SID_AND_ATTRIBUTES, SetTokenInformation,
    TOKEN_ADJUST_DEFAULT, TOKEN_DUPLICATE, TOKEN_MANDATORY_LABEL, TOKEN_QUERY, TokenIntegrityLevel,
    WELL_KNOWN_SID_TYPE, WinBuiltinAdministratorsSid,
};
use windows_sys::Win32::System::SystemServices::SE_GROUP_INTEGRITY;
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

pub(super) fn restricted_token(integrity: WELL_KNOWN_SID_TYPE) -> io::Result<OwnedHandle> {
    let mut token = ptr::null_mut();
    if unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ADJUST_DEFAULT,
            &mut token,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let token = unsafe { OwnedHandle::from_raw_handle(token) };
    let (mut admin, _) = sid(WinBuiltinAdministratorsSid)?;
    let disabled = SID_AND_ATTRIBUTES {
        Sid: admin.as_mut_ptr().cast(),
        Attributes: 0,
    };
    let mut restricted = ptr::null_mut();
    if unsafe {
        CreateRestrictedToken(
            token.as_raw_handle(),
            DISABLE_MAX_PRIVILEGE,
            1,
            &disabled,
            0,
            ptr::null(),
            0,
            ptr::null(),
            &mut restricted,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let restricted = unsafe { OwnedHandle::from_raw_handle(restricted) };
    let (label_sid, sid_size) = sid(integrity)?;
    let total = size_of::<TOKEN_MANDATORY_LABEL>() + sid_size as usize;
    let mut buffer = vec![0usize; total.div_ceil(size_of::<usize>())];
    let destination = unsafe {
        buffer
            .as_mut_ptr()
            .cast::<u8>()
            .add(size_of::<TOKEN_MANDATORY_LABEL>())
    };
    unsafe {
        ptr::copy_nonoverlapping(
            label_sid.as_ptr().cast::<u8>(),
            destination,
            sid_size as usize,
        );
        buffer
            .as_mut_ptr()
            .cast::<TOKEN_MANDATORY_LABEL>()
            .write(TOKEN_MANDATORY_LABEL {
                Label: SID_AND_ATTRIBUTES {
                    Sid: destination.cast(),
                    Attributes: SE_GROUP_INTEGRITY as u32,
                },
            });
    }
    if unsafe {
        SetTokenInformation(
            restricted.as_raw_handle(),
            TokenIntegrityLevel,
            buffer.as_ptr().cast(),
            total as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let mut required = 0;
    unsafe {
        GetTokenInformation(
            restricted.as_raw_handle(),
            TokenIntegrityLevel,
            ptr::null_mut(),
            0,
            &mut required,
        )
    };
    if required == 0 || required > 1024 * 1024 {
        return Err(io::Error::last_os_error());
    }
    let mut actual = vec![0usize; (required as usize).div_ceil(size_of::<usize>())];
    if unsafe {
        GetTokenInformation(
            restricted.as_raw_handle(),
            TokenIntegrityLevel,
            actual.as_mut_ptr().cast(),
            required,
            &mut required,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let actual = unsafe { &*actual.as_ptr().cast::<TOKEN_MANDATORY_LABEL>() };
    if unsafe { EqualSid(actual.Label.Sid, label_sid.as_ptr().cast_mut().cast()) } == 0 {
        return Err(io::Error::other(
            "restricted token integrity level did not change",
        ));
    }
    Ok(restricted)
}

fn sid(kind: WELL_KNOWN_SID_TYPE) -> io::Result<(Vec<usize>, u32)> {
    let mut size = SECURITY_MAX_SID_SIZE;
    let mut value = vec![0usize; (size as usize).div_ceil(size_of::<usize>())];
    if unsafe { CreateWellKnownSid(kind, ptr::null_mut(), value.as_mut_ptr().cast(), &mut size) }
        == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok((value, size))
}
