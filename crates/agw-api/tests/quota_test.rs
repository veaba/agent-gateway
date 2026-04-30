//! Integration tests for quota endpoints.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use agw_api::handlers::create_router;

#[tokio::test]
async fn test_quota_status_empty() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/quota")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    let quotas = json["data"]["quotas"].as_array().unwrap();
    assert!(quotas.is_empty());
}

#[tokio::test]
async fn test_set_quota_plan_not_found() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let payload = serde_json::json!({
        "daily": 1000,
        "monthly": 10000,
        "rpm": 60
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/quota/nonexistent-plan")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
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
async fn test_set_quota_success() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Create a plan first
    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "Quota Test Plan",
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

    // Set quota
    let quota_payload = serde_json::json!({
        "daily": 5000,
        "monthly": 50000,
        "rpm": 120
    });

    let quota_resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/quota/{}", plan_id))
                .header("content-type", "application/json")
                .body(Body::from(quota_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(quota_resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(quota_resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["planId"], plan_id);
    assert_eq!(json["data"]["limits"]["daily"], 5000);
    assert_eq!(json["data"]["limits"]["monthly"], 50000);
    assert_eq!(json["data"]["limits"]["rpm"], 120);
}

#[tokio::test]
async fn test_quota_status_with_plan_filter() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Create a plan first
    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "Filter Plan",
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

    // Query with filter
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/quota?planId={}", plan_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    let quotas = json["data"]["quotas"].as_array().unwrap();
    assert_eq!(quotas.len(), 1);
    assert_eq!(quotas[0]["planId"], plan_id);
}
