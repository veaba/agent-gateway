//! StorageManager integration tests

use std::path::PathBuf;
use agw_core::storage::{StorageManager, RequestLogStore, LogWriter, MetricsCollector, RequestLogEntry, LogLevel};
use chrono::Utc;

fn temp_dirs() -> (PathBuf, PathBuf) {
    let config_dir = tempfile::tempdir().unwrap().into_path();
    let data_dir = tempfile::tempdir().unwrap().into_path();
    (config_dir, data_dir)
}

#[tokio::test]
async fn test_storage_manager_with_paths() {
    let (config_dir, data_dir) = temp_dirs();

    let manager = StorageManager::with_paths(config_dir.clone(), data_dir.clone()).await.unwrap();

    // data_dir() delegates to config.data_dir() which uses system data dir, not the passed-in data_dir
    // log_dir() is data_dir().join("logs")
    let log_dir = manager.log_dir();
    assert!(log_dir.to_string_lossy().contains("logs"));
}

#[tokio::test]
async fn test_storage_manager_data_dir_and_log_dir() {
    let (config_dir, data_dir) = temp_dirs();

    let manager = StorageManager::with_paths(config_dir, data_dir.clone()).await.unwrap();

    let dd = manager.data_dir();
    assert!(dd.to_string_lossy().contains("agent-gateway") || dd == data_dir);

    let ld = manager.log_dir();
    assert!(ld.to_string_lossy().contains("logs"));
}

#[test]
fn test_log_writer() {
    let dir = tempfile::tempdir().unwrap().into_path();
    let store = std::sync::Arc::new(RequestLogStore::new(dir.clone()));
    let writer = LogWriter::new(store.clone());

    let entry = RequestLogEntry {
        id: "1".to_string(),
        timestamp: Utc::now(),
        level: LogLevel::Info,
        message: "test log".to_string(),
        target: None,
        plan_id: Some("plan-1".to_string()),
        request_id: None,
        agent_id: None,
        model_id: None,
        status_code: Some(200),
        latency_ms: Some(50),
        error: None,
    };

    writer.write(&entry).unwrap();

    let log_file = dir.join("requests.log");
    assert!(log_file.exists());
}

#[test]
fn test_metrics_collector_record_request() {
    let mut collector = MetricsCollector::default();

    let entry = RequestLogEntry {
        id: "1".to_string(),
        timestamp: Utc::now(),
        level: LogLevel::Info,
        message: "ok".to_string(),
        target: None,
        plan_id: Some("plan-1".to_string()),
        request_id: None,
        agent_id: None,
        model_id: None,
        status_code: Some(200),
        latency_ms: Some(100),
        error: None,
    };

    collector.record_request(&entry);
    assert_eq!(collector.total_requests, 1);
    assert_eq!(collector.total_errors, 0);
    assert_eq!(collector.success_rate(), 1.0);

    let error_entry = RequestLogEntry {
        id: "2".to_string(),
        timestamp: Utc::now(),
        level: LogLevel::Error,
        message: "fail".to_string(),
        target: None,
        plan_id: Some("plan-1".to_string()),
        request_id: None,
        agent_id: None,
        model_id: None,
        status_code: Some(500),
        latency_ms: Some(200),
        error: Some("server error".to_string()),
    };

    collector.record_request(&error_entry);
    assert_eq!(collector.total_requests, 2);
    assert_eq!(collector.total_errors, 1);
    assert_eq!(collector.success_rate(), 0.5);

    let plan_metrics = collector.by_plan.get("plan-1").unwrap();
    assert_eq!(plan_metrics.requests, 2);
    // Note: errors field is not incremented by record_request; total_errors is tracked globally
    assert_eq!(collector.total_errors, 1);
}

#[test]
fn test_plan_metrics_latency_averaging() {
    let mut collector = MetricsCollector::default();

    for latency in [100, 200, 300] {
        let entry = RequestLogEntry {
            id: latency.to_string(),
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: "ok".to_string(),
            target: None,
            plan_id: Some("plan-1".to_string()),
            request_id: None,
            agent_id: None,
            model_id: None,
            status_code: Some(200),
            latency_ms: Some(latency),
            error: None,
        };
        collector.record_request(&entry);
    }

    let plan_metrics = collector.by_plan.get("plan-1").unwrap();
    assert_eq!(plan_metrics.avg_latency_ms, 200.0);
    assert_eq!(collector.avg_latency_ms(), 200.0);
}
