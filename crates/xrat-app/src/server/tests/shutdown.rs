use crate::app::tests::{TestAppBuilder, signals::FixtureShutdown};
use std::sync::{Arc, atomic::Ordering};
use std::time::Duration;

#[tokio::test]
async fn injected_shutdown_and_registration_failure_stop_the_server() {
    for fail in [false, true] {
        let (context, _root) = TestAppBuilder::new("server-shutdown")
            .build_with_root()
            .await;
        let mut settings = context.app_config.server.clone();
        settings.host = "127.0.0.1".into();
        settings.port = 0;
        let signal = Arc::new(FixtureShutdown {
            fail,
            ..Default::default()
        });
        let registered = signal.registered.notified();
        let server = crate::server::serve_with_signal(
            context.db.clone(),
            &settings,
            &context.app_config.routing,
            signal.clone(),
        );
        tokio::pin!(server);
        tokio::select! {
            _ = registered => {},
            result = &mut server => panic!("server stopped before shutdown registration: {result:?}"),
            _ = tokio::time::sleep(Duration::from_secs(2)) => panic!("server did not register shutdown"),
        }
        signal.shutdown.notify_one();
        tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(signal.calls.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn daemon_shutdown_channel_remains_supported() {
    let (context, _root) = TestAppBuilder::new("server-daemon-shutdown")
        .build_with_root()
        .await;
    let mut settings = context.app_config.server.clone();
    settings.host = "127.0.0.1".into();
    settings.port = 0;
    let (sender, receiver) = tokio::sync::oneshot::channel();
    sender.send(()).unwrap();
    tokio::time::timeout(
        Duration::from_secs(2),
        crate::server::serve_with_shutdown(
            context.db.clone(),
            &settings,
            &context.app_config.routing,
            receiver,
        ),
    )
    .await
    .unwrap()
    .unwrap();
}
