use super::*;
use std::os::windows::io::AsRawHandle;
use windows_sys::Win32::Security::{
    ImpersonateLoggedOnUser, RevertToSelf, WinLowLabelSid, WinMediumLabelSid,
};

#[path = "windows_test_token.rs"]
mod token;

pub(crate) struct Impersonation(std::marker::PhantomData<std::rc::Rc<()>>);
impl Drop for Impersonation {
    fn drop(&mut self) {
        assert_ne!(
            unsafe { RevertToSelf() },
            0,
            "failed to restore the test thread token"
        );
    }
}

pub(crate) fn impersonate_owner_without_admin() -> io::Result<Impersonation> {
    let token = token::restricted_token(WinMediumLabelSid)?;
    if unsafe { ImpersonateLoggedOnUser(token.as_raw_handle()) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(Impersonation(std::marker::PhantomData))
}

#[tokio::test]
async fn ordinary_daemon_owner_can_replace_listener_instances_and_exchange_data() {
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
    let impersonation = impersonate_owner_without_admin().unwrap();
    drop(listener(&endpoint, false).expect("ordinary daemon owner must replace its listener"));
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

#[tokio::test]
async fn low_integrity_owner_cannot_write_to_the_medium_integrity_pipe() {
    let endpoint = std::path::PathBuf::from(format!(
        r"\\.\pipe\xrat-low-owner-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _server = listener(&endpoint, true).unwrap();
    let token = token::restricted_token(WinLowLabelSid).unwrap();
    assert_ne!(unsafe { ImpersonateLoggedOnUser(token.as_raw_handle()) }, 0);
    let guard = Impersonation(std::marker::PhantomData);
    let denied = connect(&endpoint).err();
    drop(guard);
    assert_eq!(
        denied
            .expect("low integrity client must be denied")
            .raw_os_error(),
        Some(5)
    );
}
