use super::*;

#[tokio::test]
async fn builder_assigns_isolated_ports_and_valid_paths() {
    let context = TestAppBuilder::new("builder-ports").build().await;
    let expected = DEFAULT_PORT_BASE;
    assert!(context.app_config.runtime.socks.port >= expected);
    assert_eq!(
        context.app_config.runtime.http.port,
        context.app_config.runtime.socks.port + 1
    );
    assert!(context.runtime_paths.database_path.exists());
}

#[tokio::test]
async fn default_ports_keeps_app_config_defaults() {
    let context = TestAppBuilder::new("builder-defaults")
        .with_default_ports()
        .build()
        .await;
    assert_eq!(
        context.app_config.runtime.socks.port,
        AppConfig::default().runtime.socks.port
    );
}

#[tokio::test]
async fn build_with_root_returns_live_tempdir() {
    let (context, root) = TestAppBuilder::new("builder-root").build_with_root().await;
    assert!(root.path().exists());
    assert!(context.runtime_paths.root_dir.starts_with(root.path()));
}

#[tokio::test]
async fn from_parts_builds_context_without_cli() {
    let (context, _root) = TestAppBuilder::new("builder-parts").build_with_root().await;
    let rebuilt = AppContext::from_parts(context.runtime_paths.clone(), context.app_config.clone())
        .await
        .expect("context should build from parts");
    assert_eq!(
        rebuilt.runtime_paths.database_path,
        context.runtime_paths.database_path
    );
}
