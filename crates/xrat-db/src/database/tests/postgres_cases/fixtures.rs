use super::*;

pub(in super::super) async fn verify_database_backend(db: &Database) {
    let (first_id, second_id) = config_cases::verify_import_and_config_state(db).await;
    config_cases::verify_reconcile_state(db).await;
    connection_test_cases::verify_connection_test_state(db, second_id).await;
    runtime_session_cases::verify_runtime_session_state(db, second_id).await;
    super::super::geoip_cache_cases::verify_geoip_cache_state(db).await;

    let deleted = db
        .get_config_by_id(first_id)
        .await
        .expect("deleted query")
        .expect("soft-deleted config remains stored");
    assert!(deleted.is_deleted);
    assert!(!deleted.is_active);
    db.hard_delete_config(first_id).await.expect("purge config");
    assert!(
        db.get_config_by_id(first_id)
            .await
            .expect("purged query")
            .is_none()
    );
}
