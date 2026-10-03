use super::*;

#[test]
fn resolves_default_target_dir_from_mmdb_config() {
    let context = test_context();

    assert_eq!(
        resolve_mmdb_target_dir(&context, None),
        context.runtime_paths.root_dir.join("mmdb")
    );
}

#[test]
fn resolves_override_target_dir_verbatim() {
    let context = test_context();

    assert_eq!(
        resolve_mmdb_target_dir(&context, Some(&PathBuf::from("./tmp/mmdb"))),
        PathBuf::from("./tmp/mmdb")
    );
}

fn test_context() -> AppContext {
    tokio::runtime::Runtime::new()
        .expect("runtime should build")
        .block_on(crate::app::tests::TestAppBuilder::new("geoip").build())
}
