//! QuotaTracker integration tests

use std::sync::Arc;
use agw_core::business::quota::{QuotaTracker, QuotaLimit};
use agw_core::storage::SqliteStore;

#[tokio::test]
async fn test_quota_tracker_check_and_consume() {
    let tracker = QuotaTracker::new();

    tracker.set_limits("plan-1", QuotaLimit {
        daily: Some(10),
        monthly: Some(100),
        rpm: Some(5),
    }).await;

    // Consume within limit
    for i in 0..10 {
        let allowed = tracker.check_and_consume("plan-1").await;
        assert!(allowed, "Should allow consumption #{} within daily limit", i + 1);
    }

    // Should exceed daily limit
    let allowed = tracker.check_and_consume("plan-1").await;
    assert!(!allowed, "Should exceed daily limit");
}

#[tokio::test]
async fn test_quota_tracker_check_and_consume_no_limits() {
    let tracker = QuotaTracker::new();

    // No limits set - should allow unlimited consumption
    for _ in 0..100 {
        let allowed = tracker.check_and_consume("plan-1").await;
        assert!(allowed);
    }
}

#[tokio::test]
async fn test_quota_tracker_get_usage() {
    let tracker = QuotaTracker::new();

    tracker.set_limits("plan-1", QuotaLimit {
        daily: Some(10),
        monthly: Some(100),
        rpm: None,
    }).await;

    let usage_before = tracker.get_usage("plan-1").await;
    assert!(usage_before.is_none() || usage_before.unwrap().daily_used == 0);

    tracker.check_and_consume("plan-1").await;

    let usage_after = tracker.get_usage("plan-1").await;
    assert!(usage_after.is_some());
    let usage = usage_after.unwrap();
    assert_eq!(usage.daily_used, 1);
    assert_eq!(usage.monthly_used, 1);
    assert_eq!(usage.rpm_used, 1);
}

#[tokio::test]
async fn test_quota_tracker_get_limits() {
    let tracker = QuotaTracker::new();

    let limits = QuotaLimit {
        daily: Some(10),
        monthly: Some(100),
        rpm: Some(5),
    };
    tracker.set_limits("plan-1", limits.clone()).await;

    let retrieved = tracker.get_limits("plan-1").await;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.daily, Some(10));
    assert_eq!(retrieved.monthly, Some(100));
    assert_eq!(retrieved.rpm, Some(5));

    let not_found = tracker.get_limits("plan-2").await;
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_quota_tracker_reset() {
    let tracker = QuotaTracker::new();

    tracker.set_limits("plan-1", QuotaLimit {
        daily: Some(10),
        monthly: Some(100),
        rpm: Some(5),
    }).await;

    for _ in 0..5 {
        tracker.check_and_consume("plan-1").await;
    }

    let usage = tracker.get_usage("plan-1").await.unwrap();
    assert_eq!(usage.daily_used, 5);

    tracker.reset("plan-1").await;

    let usage_after = tracker.get_usage("plan-1").await.unwrap();
    assert_eq!(usage_after.daily_used, 0);
    assert_eq!(usage_after.monthly_used, 0);
    assert_eq!(usage_after.rpm_used, 0);
}

#[tokio::test]
async fn test_quota_tracker_get_usage_percent() {
    let tracker = QuotaTracker::new();

    tracker.set_limits("plan-1", QuotaLimit {
        daily: Some(10),
        monthly: Some(100),
        rpm: Some(5),
    }).await;

    // No usage yet
    let pct = tracker.get_usage_percent("plan-1").await;
    assert!(pct.is_none(), "No usage record yet");

    // Consume some
    for _ in 0..5 {
        tracker.check_and_consume("plan-1").await;
    }

    let pct = tracker.get_usage_percent("plan-1").await;
    assert!(pct.is_some());
    let (daily_pct, monthly_pct, rpm_pct) = pct.unwrap();
    assert_eq!(daily_pct, 0.5);
    assert_eq!(monthly_pct, 0.05);
    assert_eq!(rpm_pct, 1.0);
}

#[tokio::test]
async fn test_quota_tracker_get_usage_percent_no_limits() {
    let tracker = QuotaTracker::new();

    tracker.check_and_consume("plan-1").await;

    let pct = tracker.get_usage_percent("plan-1").await;
    assert!(pct.is_none(), "No limits set, should return None");
}

#[tokio::test]
async fn test_quota_tracker_sqlite_persistence() {
    let sqlite_store = Arc::new(SqliteStore::in_memory().unwrap());
    let tracker = QuotaTracker::with_sqlite(sqlite_store);

    tracker.set_limits("plan-1", QuotaLimit {
        daily: Some(10),
        monthly: Some(100),
        rpm: Some(5),
    }).await;

    // Consume some quota
    for _ in 0..3 {
        tracker.check_and_consume("plan-1").await;
    }

    let usage = tracker.get_usage("plan-1").await.unwrap();
    assert_eq!(usage.daily_used, 3);
    assert_eq!(usage.monthly_used, 3);
}

#[tokio::test]
async fn test_quota_tracker_load_from_sqlite() {
    let sqlite_store = Arc::new(SqliteStore::in_memory().unwrap());
    let tracker = QuotaTracker::with_sqlite(sqlite_store.clone());

    tracker.set_limits("plan-1", QuotaLimit {
        daily: Some(10),
        monthly: Some(100),
        rpm: Some(5),
    }).await;

    // Consume and persist
    for _ in 0..5 {
        tracker.check_and_consume("plan-1").await;
    }

    // Load from SQLite into a new tracker
    let tracker2 = QuotaTracker::with_sqlite(sqlite_store);
    tracker2.load_from_sqlite(&["plan-1".to_string()]).await.unwrap();

    let usage = tracker2.get_usage("plan-1").await;
    assert!(usage.is_some());
    let usage = usage.unwrap();
    assert_eq!(usage.daily_used, 5);
    assert_eq!(usage.monthly_used, 5);
}

#[tokio::test]
async fn test_quota_tracker_multiple_plans() {
    let tracker = QuotaTracker::new();

    tracker.set_limits("plan-1", QuotaLimit {
        daily: Some(10),
        monthly: Some(100),
        rpm: None,
    }).await;

    tracker.set_limits("plan-2", QuotaLimit {
        daily: Some(20),
        monthly: Some(200),
        rpm: None,
    }).await;

    for _ in 0..5 {
        tracker.check_and_consume("plan-1").await;
        tracker.check_and_consume("plan-2").await;
    }

    let usage1 = tracker.get_usage("plan-1").await.unwrap();
    let usage2 = tracker.get_usage("plan-2").await.unwrap();

    assert_eq!(usage1.daily_used, 5);
    assert_eq!(usage2.daily_used, 5);
}
