//! RequestLogStore integration tests

use std::path::PathBuf;
use agw_core::storage::{RequestLogStore, RequestLogEntry, LogLevel};
use chrono::Utc;

fn temp_log_dir() -> PathBuf {
    tempfile::tempdir().unwrap().into_path()
}

fn make_entry(id: &str, level: LogLevel, plan_id: Option<&str>) -> RequestLogEntry {
    RequestLogEntry {
        id: id.to_string(),
        timestamp: Utc::now(),
        level,
        message: format!("message {}", id),
        target: Some("test".to_string()),
        plan_id: plan_id.map(|s| s.to_string()),
        request_id: Some(format!("req-{}", id)),
        agent_id: None,
        model_id: Some("model-1".to_string()),
        status_code: Some(200),
        latency_ms: Some(100),
        error: None,
    }
}

#[test]
fn test_request_log_store_new() {
    let dir = temp_log_dir();
    let store = RequestLogStore::new(dir.clone());
    assert_eq!(store.log_dir(), &dir);
}

#[test]
fn test_write_sync() {
    let dir = temp_log_dir();
    let store = RequestLogStore::new(dir.clone());

    let entry = make_entry("1", LogLevel::Info, Some("plan-1"));
    store.write_sync(&entry).unwrap();

    let log_file = dir.join("requests.log");
    assert!(log_file.exists());

    let content = std::fs::read_to_string(&log_file).unwrap();
    assert!(content.contains("message 1"));
    assert!(content.contains("plan-1"));
}

#[tokio::test]
async fn test_write_async() {
    let dir = temp_log_dir();
    let store = RequestLogStore::new(dir.clone());

    let entry = make_entry("1", LogLevel::Info, Some("plan-1"));
    store.write(&entry).await.unwrap();

    let log_file = dir.join("requests.log");
    assert!(log_file.exists());
}

#[tokio::test]
async fn test_read_with_filters() {
    let dir = temp_log_dir();
    let store = RequestLogStore::new(dir.clone());

    // Write entries with different levels and plan_ids
    store.write(&make_entry("1", LogLevel::Info, Some("plan-a"))).await.unwrap();
    store.write(&make_entry("2", LogLevel::Error, Some("plan-a"))).await.unwrap();
    store.write(&make_entry("3", LogLevel::Info, Some("plan-b"))).await.unwrap();
    store.write(&make_entry("4", LogLevel::Warn, None)).await.unwrap();

    // Read all
    let all = store.read(100, 0, None, None).await.unwrap();
    assert_eq!(all.len(), 4);

    // Filter by level
    let errors = store.read(100, 0, Some(LogLevel::Error), None).await.unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].id, "2");

    // Filter by plan_id
    let plan_a = store.read(100, 0, None, Some("plan-a".to_string())).await.unwrap();
    assert_eq!(plan_a.len(), 2);
}

#[tokio::test]
async fn test_count() {
    let dir = temp_log_dir();
    let store = RequestLogStore::new(dir.clone());

    store.write(&make_entry("1", LogLevel::Info, Some("plan-a"))).await.unwrap();
    store.write(&make_entry("2", LogLevel::Error, Some("plan-a"))).await.unwrap();
    store.write(&make_entry("3", LogLevel::Info, Some("plan-b"))).await.unwrap();

    let total = store.count(None, None).await.unwrap();
    assert_eq!(total, 3);

    let error_count = store.count(Some(LogLevel::Error), None).await.unwrap();
    assert_eq!(error_count, 1);

    let plan_a_count = store.count(None, Some("plan-a".to_string())).await.unwrap();
    assert_eq!(plan_a_count, 2);
}

#[test]
fn test_rotate_file_sync() {
    let dir = temp_log_dir();
    let store = RequestLogStore::with_config(dir.clone(), 100, 5, false);

    // Write enough data to trigger rotation
    for i in 0..10 {
        let entry = make_entry(&format!("{}", i), LogLevel::Info, None);
        store.write_sync(&entry).unwrap();
    }

    // Check that rotated files exist
    let files: Vec<_> = std::fs::read_dir(&dir).unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    assert!(files.iter().any(|f| f.starts_with("requests_") && f.ends_with(".log")));
}

#[test]
fn test_cleanup_old_files() {
    let dir = temp_log_dir();
    let store = RequestLogStore::with_config(dir.clone(), 50, 2, false);

    // Trigger multiple rotations
    for i in 0..5 {
        let entry = make_entry(&format!("{}", i), LogLevel::Info, None);
        store.write_sync(&entry).unwrap();
        // Force rotation by writing a large entry
        let big_entry = RequestLogEntry {
            id: format!("big-{}", i),
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: "x".repeat(100),
            target: None,
            plan_id: None,
            request_id: None,
            agent_id: None,
            model_id: None,
            status_code: None,
            latency_ms: None,
            error: None,
        };
        store.write_sync(&big_entry).unwrap();
    }

    let files: Vec<_> = std::fs::read_dir(&dir).unwrap()
        .filter_map(|e| e.ok())
        .collect();

    // Should have at most max_files + current file
    assert!(files.len() <= 3);
}
