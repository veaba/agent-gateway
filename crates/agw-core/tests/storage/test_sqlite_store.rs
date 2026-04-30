//! SqliteStore integration tests

use agw_core::storage::SqliteStore;
use agw_core::storage::sqlite::RequestLogParams;

#[test]
fn test_sqlite_store_in_memory() {
    let _store = SqliteStore::in_memory().unwrap();
    // If init_schema succeeds, tables are created
    assert!(true);
}

#[test]
fn test_sqlite_store_new_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let store = SqliteStore::new(path.clone()).unwrap();
    assert!(path.exists());
    drop(store);
    // Cleanup handled by tempdir
}

#[tokio::test]
async fn test_log_request() {
    let store = SqliteStore::in_memory().unwrap();

    let params = RequestLogParams {
        request_id: "req-1".to_string(),
        plan_id: "plan-1".to_string(),
        agent_id: Some("agent-1".to_string()),
        model_id: "model-1".to_string(),
        input_tokens: Some(100),
        output_tokens: Some(50),
        status_code: Some(200),
        error_message: None,
        latency_ms: Some(120),
    };

    store.log_request(params).await.unwrap();

    let logs = store.get_recent_logs(10).await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].request_id, "req-1");
    assert_eq!(logs[0].plan_id, "plan-1");
    assert_eq!(logs[0].model_id, "model-1");
    assert_eq!(logs[0].status_code, Some(200));
    assert_eq!(logs[0].latency_ms, Some(120));
}

#[tokio::test]
async fn test_log_health_check() {
    let store = SqliteStore::in_memory().unwrap();

    store.log_health_check(
        "plan-1".to_string(),
        "healthy".to_string(),
        Some(45),
        None,
    ).await.unwrap();

    let history = store.get_health_history(Some("plan-1".to_string()), 10).await.unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].plan_id, "plan-1");
    assert_eq!(history[0].status, "healthy");
    assert_eq!(history[0].response_time_ms, Some(45));
}

#[tokio::test]
async fn test_get_recent_logs_limit() {
    let store = SqliteStore::in_memory().unwrap();

    for i in 0..5 {
        let params = RequestLogParams {
            request_id: format!("req-{}", i),
            plan_id: "plan-1".to_string(),
            agent_id: None,
            model_id: "model-1".to_string(),
            input_tokens: None,
            output_tokens: None,
            status_code: Some(200),
            error_message: None,
            latency_ms: Some(100),
        };
        store.log_request(params).await.unwrap();
    }

    let logs = store.get_recent_logs(3).await.unwrap();
    assert_eq!(logs.len(), 3);
}

#[tokio::test]
async fn test_concurrent_access() {
    let store = SqliteStore::in_memory().unwrap();
    let mut handles = vec![];

    for i in 0..10 {
        let store_clone = store.clone();
        let handle = tokio::spawn(async move {
            let params = RequestLogParams {
                request_id: format!("req-{}", i),
                plan_id: "plan-1".to_string(),
                agent_id: None,
                model_id: "model-1".to_string(),
                input_tokens: None,
                output_tokens: None,
                status_code: Some(200),
                error_message: None,
                latency_ms: Some(100),
            };
            store_clone.log_request(params).await.unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let logs = store.get_recent_logs(100).await.unwrap();
    assert_eq!(logs.len(), 10);
}
