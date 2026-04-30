//! Integration tests for fallback endpoints.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use agw_api::handlers::create_router;

#[tokio::test]
async fn test_get_fallback_default() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/fallback")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    // Default FallbackConfig has enabled=true, max_attempts=3, empty priority_order
    assert!(json["data"]["enabled"].as_bool().unwrap());
    assert_eq!(json["data"]["maxAttempts"], 3);
    let priority = json["data"]["priorityOrder"].as_array().unwrap();
    assert!(priority.is_empty());
}

#[tokio::test]
async fn test_update_fallback_success() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let payload = serde_json::json!({
        "enabled": false,
        "maxAttempts": 5,
        "priorityOrder": ["plan-1", "plan-2"]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/fallback")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert!(!json["data"]["enabled"].as_bool().unwrap());
    assert_eq!(json["data"]["maxAttempts"], 5);
    let priority = json["data"]["priorityOrder"].as_array().unwrap();
    assert_eq!(priority.len(), 2);
    assert_eq!(priority[0], "plan-1");
    assert_eq!(priority[1], "plan-2");
}

#[tokio::test]
async fn test_update_fallback_partial() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Update only enabled
    let payload = serde_json::json!({
        "enabled": false
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/fallback")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert!(!json["data"]["enabled"].as_bool().unwrap());
    // maxAttempts should remain default
    assert_eq!(json["data"]["maxAttempts"], 3);
}
