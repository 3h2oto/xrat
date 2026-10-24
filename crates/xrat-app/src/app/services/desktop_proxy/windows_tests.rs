use super::*;
use crate::app::services::desktop_proxy::DesktopProxy;

struct Restore(Settings);

impl Drop for Restore {
    fn drop(&mut self) {
        WinInet
            .write(&self.0)
            .expect("restore runner WinINET settings");
    }
}

#[test]
#[ignore = "changes WinINET settings; requires an isolated Windows acceptance runner"]
fn native_wininet_restores_owned_settings_and_preserves_conflicts() {
    let backend = WinInet;
    let original = backend.read().unwrap();
    let restore = Restore(original.clone());
    let directory = tempfile::tempdir().unwrap();
    let proxy = DesktopProxy::open(&backend, directory.path()).unwrap();
    let manual = Settings {
        flags: 1 | 2,
        server: Some("http=127.0.0.1:17891;https=127.0.0.1:17891".into()),
        bypass: Some("localhost;127.*;<local>".into()),
        pac: None,
    };
    let pac = Settings {
        flags: 1 | 4,
        server: None,
        bypass: Some("localhost;127.*;<local>".into()),
        pac: Some("http://127.0.0.1:17892/proxy.pac".into()),
    };
    proxy.enable(manual.clone()).unwrap();
    assert_eq!(backend.read().unwrap(), manual);
    assert_eq!(proxy.status().unwrap(), "XRAT-managed");
    proxy.enable(pac.clone()).unwrap();
    assert_eq!(backend.read().unwrap(), pac);
    proxy.disable().unwrap();
    proxy.disable().unwrap();
    assert_eq!(backend.read().unwrap(), original);

    proxy.enable(manual.clone()).unwrap();
    backend.write(&pac).unwrap();
    assert!(proxy.disable().is_err());
    assert_eq!(backend.read().unwrap(), pac);
    assert!(directory.path().join("desktop-proxy.json").is_file());
    backend.write(&manual).unwrap();
    proxy.disable().unwrap();
    assert_eq!(backend.read().unwrap(), original);
    drop(restore);
}
