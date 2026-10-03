use super::*;

#[test]
fn same_version_ignores_v_prefix() {
    assert!(same_version("v0.2.1", "0.2.1"));
    assert!(same_version("0.2.1", "v0.2.1"));
    assert!(same_version(" v0.2.1 ", "0.2.1"));
    assert!(!same_version("v0.2.1", "0.2.2"));
}
