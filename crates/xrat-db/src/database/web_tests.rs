use super::*;
use crate::DatabaseConnectionConfig;

#[tokio::test]
async fn identity_sessions_tokens_and_jobs_on_sqlite_and_postgres() {
    let path = std::env::temp_dir().join(format!("xrat-web-{}.sqlite", std::process::id()));
    let sqlite = Database::connect(&DatabaseConnectionConfig::Sqlite { path: path.clone() })
        .await
        .unwrap();
    verify(&sqlite).await;
    if let Ok(url) = std::env::var("XRAT_POSTGRES_TEST_URL") {
        let postgres = Database::connect(&DatabaseConnectionConfig::Postgres {
            url,
            max_connections: 2,
            min_connections: 0,
            connect_timeout: std::time::Duration::from_secs(10),
        })
        .await
        .unwrap();
        verify(&postgres).await;
    }
    drop(sqlite);
    let _ = std::fs::remove_file(path);
}
async fn verify(db: &Database) {
    let username = format!("web-test-{}", std::process::id());
    let token = format!("token-{}", std::process::id());
    let mut user = WebUser {
        username: username.clone(),
        password_hash: "argon2-test-hash".into(),
        role: "viewer".into(),
        enabled: true,
        must_change_password: true,
        credential_version: "v1".into(),
    };
    db.web_save_user(&user).await.unwrap();
    user.role = "operator".into();
    user.credential_version = "v2".into();
    db.web_save_user(&user).await.unwrap();
    assert_eq!(
        db.web_users()
            .await
            .unwrap()
            .into_iter()
            .find(|u| u.username == username)
            .unwrap()
            .role,
        "operator"
    );
    db.web_save_session(&WebSession {
        token_hash: token.clone(),
        username: username.clone(),
        credential_version: "v2".into(),
        csrf_hash: "csrf-hash".into(),
        created_at: 100,
        accessed_at: 100,
    })
    .await
    .unwrap();
    db.web_touch_session(&token, 200).await.unwrap();
    assert_eq!(
        db.web_session(&token).await.unwrap().unwrap().accessed_at,
        200
    );
    db.web_save_token(&WebToken {
        token_hash: token.clone(),
        username: username.clone(),
        credential_version: "v2".into(),
        scope: "read".into(),
        name: "test".into(),
        created_at: 100,
    })
    .await
    .unwrap();
    assert!(db.web_token(&token).await.unwrap().is_some());
    let subscription = format!("subscription-{}", std::process::id());
    assert!(
        !db.web_save_subscription_token(&subscription, &token, "wrong-owner", "v2", 100)
            .await
            .unwrap()
    );
    assert!(
        !db.web_save_subscription_token(&subscription, &token, &username, "old-version", 100)
            .await
            .unwrap()
    );
    assert!(
        db.web_save_subscription_token(&subscription, &token, &username, "v2", 100)
            .await
            .unwrap()
    );
    assert_eq!(
        db.web_subscription_token(&subscription)
            .await
            .unwrap()
            .unwrap()
            .token_hash,
        token
    );
    assert!(db.web_token(&subscription).await.unwrap().is_none());
    db.web_revoke_user(&username).await.unwrap();
    assert!(db.web_session(&token).await.unwrap().is_none());
    assert!(db.web_tokens(&username).await.unwrap().is_empty());
    assert!(
        db.web_subscription_token(&subscription)
            .await
            .unwrap()
            .is_none()
    );
    let mut job = WebJob {
        id: token,
        username,
        kind: "test".into(),
        state: "running".into(),
        result: None,
        created_at: 100,
        updated_at: 100,
    };
    db.web_save_job(&job).await.unwrap();
    db.web_interrupt_jobs(300).await.unwrap();
    assert_eq!(
        db.web_jobs()
            .await
            .unwrap()
            .into_iter()
            .find(|j| j.id == job.id)
            .unwrap()
            .state,
        "interrupted"
    );
    job.state = "succeeded".into();
    job.updated_at = 400;
    job.result = Some("tested 2 configs".into());
    db.web_save_job(&job).await.unwrap();
    assert_eq!(
        db.web_jobs()
            .await
            .unwrap()
            .into_iter()
            .find(|j| j.id == job.id)
            .unwrap()
            .result,
        job.result
    );
    db.web_audit(&job.username, "test_action").await.unwrap();
}
