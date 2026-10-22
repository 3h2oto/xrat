use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows_sys::Win32::Foundation::{FreeLibrary, HMODULE};
use windows_sys::Win32::System::LibraryLoader::{
    GetProcAddress, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW,
};
use xrat_support::net::{SystemTunInterfaceOps, TunInterfaceOps};

struct Library(HMODULE);
impl Drop for Library {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.0) };
    }
}
type CloseAdapter = unsafe extern "system" fn(*mut c_void);
struct Adapter {
    handle: *mut c_void,
    close: CloseAdapter,
}
impl Drop for Adapter {
    fn drop(&mut self) {
        unsafe { (self.close)(self.handle) };
    }
}

pub(crate) fn verify_native_adapter(directory: &Path) {
    assert!(xrat_support::windows::is_current_process_elevated().unwrap());
    super::verify_installed(directory).unwrap();
    let path = directory.join("wintun.dll");
    super::verify_signature(&path).unwrap();
    let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let module = unsafe {
        LoadLibraryExW(
            path.as_ptr(),
            std::ptr::null_mut(),
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32,
        )
    };
    assert!(!module.is_null(), "{}", std::io::Error::last_os_error());
    let library = Library(module);
    let create =
        unsafe { GetProcAddress(library.0, c"WintunCreateAdapter".as_ptr().cast()) }.unwrap();
    let close =
        unsafe { GetProcAddress(library.0, c"WintunCloseAdapter".as_ptr().cast()) }.unwrap();
    type CreateAdapter = unsafe extern "system" fn(
        *const u16,
        *const u16,
        *const windows_sys::core::GUID,
    ) -> *mut c_void;
    let create: CreateAdapter = unsafe { std::mem::transmute(create) };
    let close: CloseAdapter = unsafe { std::mem::transmute(close) };
    let name = format!("XRAT-{}", uuid::Uuid::new_v4());
    let name_utf16: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let kind: Vec<u16> = "XRAT-Acceptance".encode_utf16().chain(Some(0)).collect();
    let ops = SystemTunInterfaceOps;
    assert!(ops.inspect_interface(&name).unwrap().is_none());
    let handle = unsafe { create(name_utf16.as_ptr(), kind.as_ptr(), std::ptr::null()) };
    assert!(!handle.is_null(), "{}", std::io::Error::last_os_error());
    let adapter = Adapter { handle, close };
    let info = ops
        .inspect_interface(&name)
        .unwrap()
        .expect("created adapter must be visible");
    assert!(info.is_tun && info.ifindex > 0);
    assert!(ops.interface_identity(&name).unwrap().is_some());
    assert!(
        ops.delete_interface(&name, info.ifindex).is_err(),
        "an open adapter must be preserved"
    );
    assert_eq!(ops.inspect_interface(&name).unwrap(), Some(info.clone()));
    drop(adapter);
    ops.delete_interface(&name, info.ifindex).unwrap();
    assert!(ops.inspect_interface(&name).unwrap().is_none());
}
