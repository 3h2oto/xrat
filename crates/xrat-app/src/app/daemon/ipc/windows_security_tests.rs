use super::*;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::ptr;

use windows_sys::Win32::Security::{
    CreateRestrictedToken, CreateWellKnownSid, DISABLE_MAX_PRIVILEGE, ImpersonateLoggedOnUser,
    RevertToSelf, SECURITY_MAX_SID_SIZE, SID_AND_ATTRIBUTES, TOKEN_DUPLICATE, TOKEN_QUERY,
    WinBuiltinAdministratorsSid,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

struct Impersonation;
impl Drop for Impersonation {
    fn drop(&mut self) {
        assert_ne!(
            unsafe { RevertToSelf() },
            0,
            "failed to restore the test thread token"
        );
    }
}

fn restricted_token() -> io::Result<OwnedHandle> {
    let mut token = ptr::null_mut();
    if unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_DUPLICATE | TOKEN_QUERY,
            &mut token,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let token = unsafe { OwnedHandle::from_raw_handle(token) };
    let mut sid_size = SECURITY_MAX_SID_SIZE;
    let mut sid = vec![0usize; (sid_size as usize).div_ceil(size_of::<usize>())];
    if unsafe {
        CreateWellKnownSid(
            WinBuiltinAdministratorsSid,
            ptr::null_mut(),
            sid.as_mut_ptr().cast(),
            &mut sid_size,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let disabled = SID_AND_ATTRIBUTES {
        Sid: sid.as_mut_ptr().cast(),
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
    Ok(unsafe { OwnedHandle::from_raw_handle(restricted) })
}

#[tokio::test]
async fn owner_without_admin_group_can_exchange_data_but_cannot_create_pipe_instances() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let endpoint = std::path::PathBuf::from(format!(
        r"\\.\pipe\xrat-owner-rights-{}-{suffix}",
        std::process::id()
    ));
    let mut server = listener(&endpoint, true).unwrap();
    let token = restricted_token().unwrap();
    assert_ne!(unsafe { ImpersonateLoggedOnUser(token.as_raw_handle()) }, 0);
    let impersonation = Impersonation;
    let broad = tokio::net::windows::named_pipe::ClientOptions::new().open(&endpoint);
    assert!(
        broad.is_err(),
        "generic write must not grant pipe-instance creation to an ordinary owner"
    );
    let mut client = connect(&endpoint).expect("ordinary owner must have read/write-data access");
    drop(impersonation);
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        server.connect().await.unwrap();
        client.write_all(b"request").await.unwrap();
        let mut request = [0; 7];
        server.read_exact(&mut request).await.unwrap();
        assert_eq!(&request, b"request");
        assert_eq!(
            xrat_support::windows::named_pipe_client_sid(&server).unwrap(),
            xrat_support::windows::current_user_sid().unwrap()
        );
        server.write_all(b"response").await.unwrap();
        let mut response = [0; 8];
        client.read_exact(&mut response).await.unwrap();
        assert_eq!(&response, b"response");
    })
    .await
    .expect("ordinary owner data exchange timed out");
}
