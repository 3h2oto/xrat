use super::super::dispatch::handle_event_inner;
use super::super::test_support::test_context;
use super::super::*;

#[tokio::test]
async fn tun_request_rejects_a_different_config_without_writing_either_file() {
    let context = test_context("tun-config-mismatch").await;
    let contents = "# daemon config\n[runtime.tun]\nenabled = true\n";
    std::fs::write(&context.runtime_paths.config_path, contents).unwrap();
    let other = context.runtime_paths.root_dir.join("other.toml");
    std::fs::write(&other, contents).unwrap();
    let mut state = SupervisorState::new("test-owner".into());
    let (tx, rx) = oneshot::channel();
    handle_event(
        &mut state,
        SupervisorEvent::RuntimeTun {
            enabled: Some(false),
            config_path: other.clone(),
            respond_to: tx,
        },
        &context,
    )
    .await;
    assert!(rx.await.unwrap().unwrap_err().contains("different config"));
    assert_eq!(
        std::fs::read_to_string(&context.runtime_paths.config_path).unwrap(),
        contents
    );
    assert_eq!(std::fs::read_to_string(other).unwrap(), contents);
}

#[tokio::test]
async fn supervisor_tun_change_updates_its_context_for_the_next_request() {
    let mut context = test_context("tun-supervisor-context").await;
    std::fs::write(
        &context.runtime_paths.config_path,
        "[runtime.tun]\nenabled = true\n",
    )
    .unwrap();
    context.app_config.runtime.tun.enabled = true;
    let mut state = SupervisorState::new("test-owner".into());
    let (tx, rx) = oneshot::channel();
    handle_event_inner(
        &mut state,
        SupervisorEvent::RuntimeTun {
            enabled: Some(false),
            config_path: context.runtime_paths.config_path.clone(),
            respond_to: tx,
        },
        &mut context,
        None,
    )
    .await;
    assert!(!rx.await.unwrap().unwrap().enabled);
    assert!(!context.app_config.runtime.tun.enabled);
    let (tx, rx) = oneshot::channel();
    handle_event_inner(
        &mut state,
        SupervisorEvent::RuntimeStatus { respond_to: tx },
        &mut context,
        None,
    )
    .await;
    let crate::app::daemon::supervisor::RuntimeStatusResult::Ok(status) = rx.await.unwrap() else {
        panic!("status failed");
    };
    let tun = status.tun.unwrap();
    assert!(!tun.enabled && !tun.active);
}
