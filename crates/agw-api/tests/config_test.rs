//! Integration tests for config endpoints.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use agw_api::handlers::create_router;

#[tokio::test]
async fn test_get_config() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert!(json["data"]["version"].as_str().unwrap().len() > 0);
    assert!(json["data"]["dataDir"].as_str().unwrap().len() > 0);
    assert!(json["data"]["configDir"].as_str().unwrap().len() > 0);
    assert_eq!(json["data"]["plansCount"], 0);
    assert!(json["data"]["providersCount"].as_u64().unwrap() > 0);
    assert_eq!(json["data"]["fallbackEnabled"], true);
    assert_eq!(json["data"]["fallbackMaxAttempts"], 3);
    assert_eq!(json["data"]["pluginsCount"], 0);
}

#[tokio::test]
async fn test_update_config_success() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let payload = serde_json::json!({
        "fallbackEnabled": false,
        "fallbackMaxAttempts": 5
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/config")
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
    assert_eq!(json["data"]["fallbackEnabled"], false);
    assert_eq!(json["data"]["fallbackMaxAttempts"], 5);
}

#[tokio::test]
async fn test_export_config_json() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/config/export")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["format"], "json");
    assert!(json["plans"].is_array());
    assert!(json["fallback"].is_object());
}

#[tokio::test]
async fn test_export_config_yaml() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/config/export?format=yaml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["format"], "yaml");
    assert!(json["content"].as_str().unwrap().len() > 0);
}

#[tokio::test]
async fn test_import_config_success() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let payload = serde_json::json!({
        "config": {
            "plans": [],
            "fallback": {
                "enabled": true,
                "max_attempts": 2,
                "priority_order": []
            }
        },
        "merge": false
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/config/import")
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
    assert_eq!(json["data"]["imported_plans"], 0);
    assert_eq!(json["data"]["imported_fallback"], true);
}

#[tokio::test]
async fn test_reset_config_success() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // First update config
    let payload = serde_json::json!({
        "fallbackEnabled": false,
        "fallbackMaxAttempts": 10
    });

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/config")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Reset config
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/config/reset")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
