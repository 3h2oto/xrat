use super::KernelInterfaceInfo;
use std::ffi::{CString, c_char, c_uint};
use std::io;

unsafe extern "C" {
    fn if_nametoindex(name: *const c_char) -> c_uint;
}

pub(super) fn inspect(name: &str) -> io::Result<Option<KernelInterfaceInfo>> {
    let name_c =
        CString::new(name).map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let index = unsafe { if_nametoindex(name_c.as_ptr()) };
    if index == 0 {
        return Ok(None);
    }
    let is_tun = name.strip_prefix("utun").is_some_and(|suffix| {
        !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
    });
    Ok(Some(KernelInterfaceInfo {
        name: name.to_string(),
        ifindex: index,
        is_tun,
    }))
}

pub(super) fn wait_for_release(name: &str, expected_ifindex: u32) -> io::Result<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    loop {
        let Some(info) = inspect(name)? else {
            return Ok(());
        };
        if !info.is_tun || info.ifindex != expected_ifindex || expected_ifindex == 0 {
            return Err(io::Error::other("utun ownership changed; refusing cleanup"));
        }
        if std::time::Instant::now() >= deadline {
            return Err(io::Error::other(
                "utun is still held by a process; ownership retained until the engine releases it",
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[cfg(test)]
#[path = "macos_tests.rs"]
mod tests;
