use super::KernelInterfaceInfo;
use std::io;
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{
    ERROR_FILE_NOT_FOUND, ERROR_INVALID_PARAMETER, ERROR_NOT_FOUND,
};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    ConvertInterfaceAliasToLuid, GetIfEntry2, MIB_IF_ROW2,
};
use windows_sys::Win32::System::Registry::{HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RegGetValueW};

fn wide(value: &str) -> io::Result<Vec<u16>> {
    if value.contains('\0') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "NUL in network identifier",
        ));
    }
    Ok(value.encode_utf16().chain(Some(0)).collect())
}

fn row(name: &str) -> io::Result<Option<MIB_IF_ROW2>> {
    let name = wide(name)?;
    if name.len() <= 1 || name.len() > 257 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid interface alias length",
        ));
    }
    let mut row = MIB_IF_ROW2::default();
    let status = unsafe { ConvertInterfaceAliasToLuid(name.as_ptr(), &mut row.InterfaceLuid) };
    match status {
        ERROR_INVALID_PARAMETER | ERROR_FILE_NOT_FOUND | ERROR_NOT_FOUND => return Ok(None),
        0 => {}
        status => return Err(io::Error::from_raw_os_error(status as i32)),
    }
    let status = unsafe { GetIfEntry2(&mut row) };
    match status {
        ERROR_FILE_NOT_FOUND | ERROR_NOT_FOUND => Ok(None),
        0 => Ok(Some(row)),
        status => Err(io::Error::from_raw_os_error(status as i32)),
    }
}

fn guid(row: &MIB_IF_ROW2) -> String {
    let guid = row.InterfaceGuid;
    format!(
        "{{{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}}}",
        guid.data1,
        guid.data2,
        guid.data3,
        guid.data4[0],
        guid.data4[1],
        guid.data4[2],
        guid.data4[3],
        guid.data4[4],
        guid.data4[5],
        guid.data4[6],
        guid.data4[7]
    )
}

fn registry_string(path: &str, value: &str) -> io::Result<Option<String>> {
    let path = wide(path)?;
    let value = wide(value)?;
    let mut buffer = [0u16; 2048];
    let mut size = size_of_val(&buffer) as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            path.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buffer.as_mut_ptr().cast(),
            &mut size,
        )
    };
    match status {
        ERROR_FILE_NOT_FOUND | ERROR_NOT_FOUND => return Ok(None),
        0 => {}
        status => return Err(io::Error::from_raw_os_error(status as i32)),
    }
    let length = size as usize / 2;
    if !size.is_multiple_of(2) || length == 0 || length > buffer.len() || buffer[length - 1] != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid network registry string",
        ));
    }
    String::from_utf16(&buffer[..length - 1])
        .map(Some)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn is_wintun(row: &MIB_IF_ROW2) -> io::Result<bool> {
    if row.Type != 53 {
        return Ok(false);
    }
    let connection = format!(
        r"SYSTEM\CurrentControlSet\Control\Network\{{4d36e972-e325-11ce-bfc1-08002be10318}}\{}\Connection",
        guid(row)
    );
    let Some(instance) = registry_string(&connection, "PnpInstanceID")? else {
        return Ok(false);
    };
    let service = registry_string(
        &format!(r"SYSTEM\CurrentControlSet\Enum\{instance}"),
        "Service",
    )?;
    Ok(service.is_some_and(|service| service.eq_ignore_ascii_case("wintun")))
}

pub(super) fn identity(name: &str) -> io::Result<Option<String>> {
    Ok(row(name)?.as_ref().map(guid))
}

pub(super) fn inspect(name: &str) -> io::Result<Option<KernelInterfaceInfo>> {
    row(name)?
        .map(|row| {
            Ok(KernelInterfaceInfo {
                name: name.to_string(),
                ifindex: row.InterfaceIndex,
                is_tun: is_wintun(&row)?,
            })
        })
        .transpose()
}

pub(super) fn wait_for_release(name: &str, expected_ifindex: u32) -> io::Result<()> {
    let expected_identity = identity(name)?;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let Some(info) = inspect(name)? else {
            return Ok(());
        };
        if expected_ifindex == 0
            || !info.is_tun
            || info.ifindex != expected_ifindex
            || identity(name)? != expected_identity
        {
            return Err(io::Error::other(
                "Windows adapter identity changed; refusing cleanup",
            ));
        }
        if Instant::now() >= deadline {
            return Err(io::Error::other(
                "Windows Wintun adapter is still present; ownership retained until the engine releases it",
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn absent_adapter_is_not_claimed_and_invalid_aliases_are_rejected() {
        assert!(super::inspect("XRAT-absent-adapter-fff").unwrap().is_none());
        assert!(
            super::identity("XRAT-absent-adapter-fff")
                .unwrap()
                .is_none()
        );
        assert!(super::inspect("bad\0alias").is_err());
    }
}
