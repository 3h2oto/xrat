use std::io;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::{Path, PathBuf};
use std::ptr;

use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::Security::Authorization::{GetSecurityInfo, SE_FILE_OBJECT};
use windows_sys::Win32::Security::{
    DACL_SECURITY_INFORMATION, EqualSid, GetSecurityDescriptorControl, GetSecurityDescriptorDacl,
    GetSecurityDescriptorOwner, OWNER_SECURITY_INFORMATION, SE_DACL_PROTECTED,
};
use windows_sys::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, CREATE_NEW, CreateDirectoryW, CreateFileW,
    FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_REPARSE_POINT,
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_GENERIC_READ,
    FILE_GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE, GetFileInformationByHandle,
    OPEN_EXISTING,
};

use super::SecurityDescriptor;

const DIRECTORY_SECURITY: &str = "O:BAG:BAD:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)";

pub fn program_data() -> io::Result<PathBuf> {
    known_folder(&windows_sys::Win32::UI::Shell::FOLDERID_ProgramData)
}

pub fn program_files() -> io::Result<PathBuf> {
    known_folder(&windows_sys::Win32::UI::Shell::FOLDERID_ProgramFiles)
}

pub fn system_directory() -> io::Result<PathBuf> {
    known_folder(&windows_sys::Win32::UI::Shell::FOLDERID_System)
}

fn known_folder(folder: &windows_sys::core::GUID) -> io::Result<PathBuf> {
    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::UI::Shell::SHGetKnownFolderPath;
    let mut value = ptr::null_mut();
    let result = unsafe { SHGetKnownFolderPath(folder, 0, ptr::null_mut(), &mut value) };
    if result < 0 {
        return Err(io::Error::other(format!(
            "Windows known-folder lookup failed: {result:#x}"
        )));
    }
    let mut length = 0;
    while unsafe { *value.add(length) } != 0 {
        length += 1;
    }
    let path = PathBuf::from(std::ffi::OsString::from_wide(unsafe {
        std::slice::from_raw_parts(value, length)
    }));
    unsafe { CoTaskMemFree(value.cast()) };
    Ok(path)
}

/// Pins a non-reparse directory with administrator ownership and write access.
pub struct ProtectedDirectory {
    path: PathBuf,
    _handle: OwnedHandle,
    reader: Option<String>,
}

impl ProtectedDirectory {
    pub fn create_file(&self, name: &str) -> io::Result<std::fs::File> {
        if name.contains([':', '/', '\\', '\0']) || name.ends_with(['.', ' ']) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected a profile file name",
            ));
        }
        let mut parts = Path::new(name).components();
        if !matches!(parts.next(), Some(std::path::Component::Normal(_))) || parts.next().is_some()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected a profile file name",
            ));
        }
        let security = SecurityDescriptor::from_sddl(&format!(
            "O:BAG:BAD:P(A;;FA;;;SY)(A;;FA;;;BA){}",
            self.reader
                .as_ref()
                .map(|reader| format!("(A;;FRFX;;;{reader})"))
                .unwrap_or_default()
        ))?;
        let attributes = security.attributes();
        let path = self.path.join(name);
        let wide = path
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                FILE_GENERIC_WRITE,
                0,
                &attributes,
                CREATE_NEW,
                FILE_ATTRIBUTE_NORMAL,
                ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        Ok(unsafe { std::fs::File::from_raw_handle(handle) })
    }

    pub fn verify_tree(&self) -> io::Result<()> {
        verify_children(&self.path, self.reader.as_deref())
    }

    pub fn create(path: &Path) -> io::Result<Self> {
        let expected = SecurityDescriptor::from_sddl(DIRECTORY_SECURITY)?;
        let attributes = expected.attributes();
        let wide = path
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        if unsafe { CreateDirectoryW(wide.as_ptr(), &attributes) } == 0 {
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::AlreadyExists {
                return Err(error);
            }
        }
        Self::open(path)
    }

    pub fn open(path: &Path) -> io::Result<Self> {
        Self::open_with_reader(path, None)
    }
    pub fn open_readable(path: &Path, reader: &str) -> io::Result<Self> {
        Self::open_with_reader(path, Some(reader))
    }
    pub fn create_readable(path: &Path, reader: &str) -> io::Result<Self> {
        validate_reader(reader)?;
        let security = SecurityDescriptor::from_sddl(&directory_security(Some(reader)))?;
        let attributes = security.attributes();
        let wide = path
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        if unsafe { CreateDirectoryW(wide.as_ptr(), &attributes) } == 0 {
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::AlreadyExists {
                return Err(error);
            }
            if let Ok(existing) = Self::open(path) {
                existing.verify_tree()?;
                set_reader(path, reader, true)?;
            }
        }
        Self::open_readable(path, reader)
    }
    pub fn make_file_readable(&self, name: &str) -> io::Result<()> {
        let reader = self.reader.as_deref().ok_or_else(denied)?;
        if !matches!(
            Path::new(name).components().next(),
            Some(std::path::Component::Normal(_))
        ) || Path::new(name).components().count() != 1
            || name.contains([':', '/', '\\', '\0'])
        {
            return Err(denied());
        }
        self.verify_tree()?;
        set_reader(&self.path.join(name), reader, false)?;
        self.verify_tree()
    }
    fn open_with_reader(path: &Path, reader: Option<&str>) -> io::Result<Self> {
        if let Some(reader) = reader {
            validate_reader(reader)?;
        }
        let wide = path
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                FILE_GENERIC_READ,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                ptr::null(),
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
                ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let handle = unsafe { OwnedHandle::from_raw_handle(handle) };
        let mut info = BY_HANDLE_FILE_INFORMATION::default();
        if unsafe { GetFileInformationByHandle(handle.as_raw_handle(), &mut info) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
            || info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        {
            return Err(denied());
        }
        let mut descriptor = ptr::null_mut();
        let result = unsafe {
            GetSecurityInfo(
                handle.as_raw_handle(),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut descriptor,
            )
        };
        if result != 0 {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        let descriptor = SecurityDescriptor(descriptor);
        verify_descriptor_with_reader(&descriptor, reader)?;
        Ok(Self {
            path: path.to_path_buf(),
            _handle: handle,
            reader: reader.map(str::to_owned),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn verify_children(path: &Path, reader: Option<&str>) -> io::Result<()> {
    use windows_sys::Win32::Security::{ACCESS_ALLOWED_ACE, GetAce, GetSecurityDescriptorDacl};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ALL_ACCESS, FILE_GENERIC_EXECUTE, FILE_GENERIC_READ, FILE_READ_ATTRIBUTES,
        READ_CONTROL,
    };
    for entry in std::fs::read_dir(path)? {
        let path = entry?.path();
        let wide = path
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                READ_CONTROL | FILE_READ_ATTRIBUTES,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                ptr::null(),
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
                ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let handle = unsafe { OwnedHandle::from_raw_handle(handle) };
        let mut info = BY_HANDLE_FILE_INFORMATION::default();
        if unsafe { GetFileInformationByHandle(handle.as_raw_handle(), &mut info) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(denied());
        }
        let mut owner = ptr::null_mut();
        let mut descriptor = ptr::null_mut();
        let result = unsafe {
            GetSecurityInfo(
                handle.as_raw_handle(),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
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
        let descriptor = SecurityDescriptor(descriptor);
        if owner.is_null()
            || !matches!(
                super::sid_string(owner)?.as_str(),
                "S-1-5-18" | "S-1-5-32-544"
            )
        {
            return Err(denied());
        }
        let mut acl = ptr::null_mut();
        let mut present = 0;
        let mut defaulted = 0;
        if unsafe {
            GetSecurityDescriptorDacl(descriptor.0, &mut present, &mut acl, &mut defaulted)
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if present == 0
            || acl.is_null()
            || !matches!(unsafe { (*acl).AceCount }, 2 | 3)
            || (reader.is_none() && unsafe { (*acl).AceCount } != 2)
        {
            return Err(denied());
        }
        let mut trustees = Vec::new();
        for index in 0..unsafe { (*acl).AceCount } {
            let mut ace = ptr::null_mut();
            if unsafe { GetAce(acl, u32::from(index), &mut ace) } == 0 {
                return Err(io::Error::last_os_error());
            }
            let ace = ace.cast::<ACCESS_ALLOWED_ACE>();
            if unsafe { (*ace).Header.AceType } != 0
                || unsafe { (*ace).Header.AceFlags } & !0x13 != 0
            {
                return Err(denied());
            }
            let sid = super::sid_string(unsafe { ptr::addr_of_mut!((*ace).SidStart).cast() })?;
            let admin = matches!(sid.as_str(), "S-1-5-18" | "S-1-5-32-544");
            if (admin && unsafe { (*ace).Mask } != FILE_ALL_ACCESS)
                || (!admin
                    && (reader != Some(sid.as_str())
                        || unsafe { (*ace).Mask } != (FILE_GENERIC_READ | FILE_GENERIC_EXECUTE)))
            {
                return Err(denied());
            }
            trustees.push(sid);
        }
        trustees.sort();
        let expected = if trustees.len() == 3 {
            let mut expected = vec![
                "S-1-5-18".to_owned(),
                "S-1-5-32-544".to_owned(),
                reader.ok_or_else(denied)?.to_owned(),
            ];
            expected.sort();
            expected
        } else {
            vec!["S-1-5-18".to_owned(), "S-1-5-32-544".to_owned()]
        };
        if trustees != expected {
            return Err(denied());
        }
        if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
            verify_children(&path, reader)?;
        }
    }
    Ok(())
}

fn denied() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "machine profile must be a non-reparse directory owned and writable only by SYSTEM/Administrators",
    )
}

#[cfg(test)]
fn verify_descriptor(actual: &SecurityDescriptor) -> io::Result<()> {
    verify_descriptor_with_reader(actual, None)
}
fn verify_descriptor_with_reader(
    actual: &SecurityDescriptor,
    reader: Option<&str>,
) -> io::Result<()> {
    let expected = SecurityDescriptor::from_sddl(&directory_security(reader))?;
    let mut control = 0;
    let mut revision = 0;
    if unsafe { GetSecurityDescriptorControl(actual.0, &mut control, &mut revision) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if control & SE_DACL_PROTECTED == 0 {
        return Err(denied());
    }
    let parts = |descriptor: &SecurityDescriptor| {
        let mut owner = ptr::null_mut();
        let mut acl = ptr::null_mut();
        let mut present = 0;
        let mut defaulted = 0;
        if unsafe { GetSecurityDescriptorOwner(descriptor.0, &mut owner, &mut defaulted) } == 0
            || unsafe {
                GetSecurityDescriptorDacl(descriptor.0, &mut present, &mut acl, &mut defaulted)
            } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if owner.is_null() || present == 0 || acl.is_null() {
            return Err(denied());
        }
        Ok((owner, acl))
    };
    let (actual_owner, actual_acl) = parts(actual)?;
    let (expected_owner, expected_acl) = parts(&expected)?;
    let acl_bytes = |acl: *const windows_sys::Win32::Security::ACL| unsafe {
        std::slice::from_raw_parts(acl.cast::<u8>(), (*acl).AclSize as usize)
    };
    if unsafe { EqualSid(actual_owner, expected_owner) } == 0
        || acl_bytes(actual_acl) != acl_bytes(expected_acl)
    {
        return Err(denied());
    }
    Ok(())
}

#[cfg(test)]
mod tests;

fn validate_reader(reader: &str) -> io::Result<()> {
    if !reader.starts_with("S-1-5-21-")
        || reader.len() > 184
        || !reader[2..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'-')
    {
        return Err(denied());
    }
    Ok(())
}
fn directory_security(reader: Option<&str>) -> String {
    format!(
        "{DIRECTORY_SECURITY}{}",
        reader
            .map(|reader| format!("(A;OICI;FRFX;;;{reader})"))
            .unwrap_or_default()
    )
}
fn set_reader(path: &Path, reader: &str, directory: bool) -> io::Result<()> {
    use windows_sys::Win32::Security::{PROTECTED_DACL_SECURITY_INFORMATION, SetFileSecurityW};
    let text = if directory {
        directory_security(Some(reader))
    } else {
        format!("O:BAG:BAD:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FRFX;;;{reader})")
    };
    let descriptor = SecurityDescriptor::from_sddl(&text)?;
    let wide = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    if unsafe {
        SetFileSecurityW(
            wide.as_ptr(),
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            descriptor.0,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
