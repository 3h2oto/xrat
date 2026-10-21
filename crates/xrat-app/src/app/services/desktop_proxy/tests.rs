use super::*;
use std::sync::Mutex;

struct Fake(Mutex<Settings>);
impl Backend for Fake {
    fn read(&self) -> Result<Settings> {
        Ok(self.0.lock().unwrap().clone())
    }
    fn write(&self, settings: &Settings) -> Result<()> {
        *self.0.lock().unwrap() = settings.clone();
        Ok(())
    }
}

fn settings(server: &str) -> Settings {
    Settings {
        flags: 2,
        server: Some(server.into()),
        bypass: Some("localhost".into()),
        pac: None,
    }
}

#[test]
fn repeated_enable_and_disable_restore_original_settings() {
    let directory = tempfile::tempdir().unwrap();
    let original = settings("original:8080");
    let backend = Fake(Mutex::new(original.clone()));
    let service = DesktopProxy::open(&backend, directory.path()).unwrap();
    service.enable(settings("xrat:1080")).unwrap();
    service.enable(settings("xrat:1081")).unwrap();
    assert_eq!(service.status().unwrap(), "XRAT-managed");
    service.disable().unwrap();
    service.disable().unwrap();
    assert_eq!(backend.read().unwrap(), original);
}

#[test]
fn refuses_to_overwrite_changes_from_another_vpn() {
    let directory = tempfile::tempdir().unwrap();
    let backend = Fake(Mutex::new(settings("original:8080")));
    let service = DesktopProxy::open(&backend, directory.path()).unwrap();
    service.enable(settings("xrat:1080")).unwrap();
    let foreign = settings("another-vpn:9090");
    backend.write(&foreign).unwrap();
    assert_eq!(service.status().unwrap(), "unrelated proxy settings");
    assert!(service.disable().is_err());
    assert!(service.enable(settings("xrat:1081")).is_err());
    assert_eq!(backend.read().unwrap(), foreign);
}
