use std::ptr;

use windows_sys::Win32::Foundation::GlobalFree;
use windows_sys::Win32::Networking::WinInet::*;

use super::{Backend, Settings};
use crate::app::Result;

pub(crate) struct WinInet;

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

unsafe fn take_string(value: *mut u16) -> Option<String> {
    if value.is_null() {
        return None;
    }
    let mut length = 0;
    while unsafe { *value.add(length) } != 0 {
        length += 1;
    }
    let text = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(value, length) });
    unsafe { GlobalFree(value.cast()) };
    (!text.is_empty()).then_some(text)
}

fn options() -> [INTERNET_PER_CONN_OPTIONW; 4] {
    [
        INTERNET_PER_CONN_FLAGS,
        INTERNET_PER_CONN_PROXY_SERVER,
        INTERNET_PER_CONN_PROXY_BYPASS,
        INTERNET_PER_CONN_AUTOCONFIG_URL,
    ]
    .map(|option| INTERNET_PER_CONN_OPTIONW {
        dwOption: option,
        Value: unsafe { std::mem::zeroed() },
    })
}

fn list(options: &mut [INTERNET_PER_CONN_OPTIONW; 4]) -> INTERNET_PER_CONN_OPTION_LISTW {
    INTERNET_PER_CONN_OPTION_LISTW {
        dwSize: size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
        pszConnection: ptr::null_mut(),
        dwOptionCount: options.len() as u32,
        dwOptionError: 0,
        pOptions: options.as_mut_ptr(),
    }
}

impl Backend for WinInet {
    fn read(&self) -> Result<Settings> {
        let mut options = options();
        options[0].dwOption = INTERNET_PER_CONN_FLAGS_UI;
        let mut list = list(&mut options);
        let mut size = size_of_val(&list) as u32;
        if unsafe {
            InternetQueryOptionW(
                ptr::null_mut(),
                INTERNET_OPTION_PER_CONNECTION_OPTION,
                (&mut list as *mut INTERNET_PER_CONN_OPTION_LISTW).cast(),
                &mut size,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
        unsafe {
            Ok(Settings {
                flags: options[0].Value.dwValue,
                server: take_string(options[1].Value.pszValue),
                bypass: take_string(options[2].Value.pszValue),
                pac: take_string(options[3].Value.pszValue),
            })
        }
    }

    fn write(&self, settings: &Settings) -> Result<()> {
        let mut options = options();
        let mut server = wide(settings.server.as_deref().unwrap_or_default());
        let mut bypass = wide(settings.bypass.as_deref().unwrap_or_default());
        let mut pac = wide(settings.pac.as_deref().unwrap_or_default());
        options[0].Value.dwValue = settings.flags;
        options[1].Value.pszValue = server.as_mut_ptr();
        options[2].Value.pszValue = bypass.as_mut_ptr();
        options[3].Value.pszValue = pac.as_mut_ptr();
        let mut list = list(&mut options);
        if unsafe {
            InternetSetOptionW(
                ptr::null_mut(),
                INTERNET_OPTION_PER_CONNECTION_OPTION,
                (&mut list as *mut INTERNET_PER_CONN_OPTION_LISTW).cast(),
                size_of_val(&list) as u32,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
        for option in [INTERNET_OPTION_SETTINGS_CHANGED, INTERNET_OPTION_REFRESH] {
            if unsafe { InternetSetOptionW(ptr::null_mut(), option, ptr::null_mut(), 0) } == 0 {
                return Err(std::io::Error::last_os_error().into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "windows_tests.rs"]
mod tests;
