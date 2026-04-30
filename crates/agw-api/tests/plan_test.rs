//! Integration tests for plan endpoints.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use agw_api::handlers::create_router;

#[tokio::test]
async fn test_list_plans_empty() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/plans")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    let plans = json["data"]["plans"].as_array().unwrap();
    assert!(plans.is_empty());
}

#[tokio::test]
async fn test_create_plan_success() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "My Alaya Plan",
        "apiKey": "sk-test-1234567890",
        "selectedModelId": "glm-5"
    });

    let response = app
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

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["name"], "My Alaya Plan");
    assert_eq!(json["data"]["providerId"], "alaya");
    assert_eq!(json["data"]["apiKeyMasked"], "sk-t...7890");
}

#[tokio::test]
async fn test_create_plan_invalid_provider() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let payload = serde_json::json!({
        "providerId": "nonexistent-provider",
        "planId": "lite",
        "name": "Bad Plan",
        "apiKey": "sk-test",
        "selectedModelId": "model"
    });

    let response = app
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(!json["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_get_plan_not_found() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/plans/does-not-exist")
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
async fn test_update_plan_success() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Create a plan first
    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "Original Name",
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

    // Update the plan
    let update_payload = serde_json::json!({
        "name": "Updated Name"
    });

    let update_resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/plans/{}", plan_id))
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
    assert_eq!(json["data"]["name"], "Updated Name");
}

#[tokio::test]
async fn test_delete_plan_success() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Create a plan first
    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "To Delete",
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

    // Delete the plan
    let delete_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/plans/{}", plan_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let get_resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/plans/{}", plan_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_set_default_plan() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Create a plan first
    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "Default Plan",
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

    // Set as default
    let default_resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/plans/{}/default", plan_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(default_resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(default_resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["id"], plan_id);
}

#[tokio::test]
async fn test_test_plan_connection() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Create a plan first
    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "Test Connection Plan",
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

    // Test connection
    let test_resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/plans/{}/test", plan_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(test_resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(test_resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["planId"], plan_id);
    assert!(json["data"]["success"].as_bool().unwrap());
}
