use super::definition::*;
use std::path::Path;

#[test]
fn machine_api_task_rejects_service_identities_and_relative_executables() {
    for sid in ["S-1-5-18", "S-1-5-32-544", "S-1-5-21-1-2-3-1001</UserId>"] {
        assert!(definition(Path::new("/protected/xrat.exe"), sid).is_err());
    }
    assert!(definition(Path::new("relative/xrat.exe"), "S-1-5-21-1-2-3-1001").is_err());
}

#[test]
fn machine_api_task_encodes_paths_without_changing_its_security_principal() {
    let executable = std::env::temp_dir()
        .join("XRAT & <machine> 'operator'")
        .join("xrat.exe");
    let xml = definition(&executable, "S-1-5-21-1-2-3-1001").unwrap();
    assert!(xml.starts_with("<?xml version=\"1.0\"?>"));
    assert!(!xml.contains("encoding="));
    assert!(xml.contains("XRAT &amp; &lt;machine&gt; &apos;operator&apos;"));
    assert!(xml.contains("<LogonType>InteractiveToken</LogonType>"));
    assert!(xml.contains("<RunLevel>LeastPrivilege</RunLevel>"));
    assert!(xml.contains("D:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FRFX;;;S-1-5-21-1-2-3-1001)"));
    assert!(!xml.contains("HighestAvailable"));
    assert!(!xml.contains("<Password>"));
    assert!(xml.contains("--profile machine serve --host 127.0.0.1"));
}
