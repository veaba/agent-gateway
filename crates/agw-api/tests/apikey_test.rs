//! Integration tests for API key endpoints.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use agw_api::handlers::create_router;

#[tokio::test]
async fn test_get_api_key_not_found() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/plans/nonexistent/key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(!json["success"].as_bool().unwrap());
    assert_eq!(json["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn test_get_api_key_success() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Create a plan first
    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "API Key Test Plan",
        "apiKey": "sk-test-1234567890",
        "selectedModelId": "glm-5"
    });

    let create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/plans")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let plan_id = json["data"]["id"].as_str().unwrap().to_string();

    // Get API key
    let key_resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/plans/{}/key", plan_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(key_resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(key_resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["plan_id"], plan_id);
    assert_eq!(json["data"]["api_key"], "sk-test-1234567890");
}

#[tokio::test]
async fn test_update_api_key_success() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Create a plan first
    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "Update Key Plan",
        "apiKey": "sk-old-key",
        "selectedModelId": "glm-5"
    });

    let create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/plans")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let plan_id = json["data"]["id"].as_str().unwrap().to_string();

    // Update API key
    let update_payload = serde_json::json!({
        "api_key": "sk-new-key-1234567890"
    });

    let update_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/plans/{}/key", plan_id))
                .header("content-type", "application/json")
                .body(Body::from(update_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(update_resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["apiKeyMasked"], "sk-n...7890");

    // Verify the key was updated
    let key_resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/plans/{}/key", plan_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(key_resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["data"]["api_key"], "sk-new-key-1234567890");
}

#[tokio::test]
async fn test_update_api_key_missing_field() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Create a plan first
    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "Missing Key Plan",
        "apiKey": "sk-test",
        "selectedModelId": "glm-5"
    });

    let create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/plans")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let plan_id = json["data"]["id"].as_str().unwrap().to_string();

    // Update with missing api_key field
    let update_payload = serde_json::json!({
        "other_field": "value"
    });

    let update_resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/plans/{}/key", plan_id))
                .header("content-type", "application/json")
                .body(Body::from(update_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(update_resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(!json["success"].as_bool().unwrap());
    assert_eq!(json["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_test_api_key_not_found() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/plans/nonexistent/key/test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(!json["success"].as_bool().unwrap());
    assert_eq!(json["error"]["code"], "NOT_FOUND");
}
