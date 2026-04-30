//! Integration tests for agent endpoints.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use agw_api::handlers::create_router;

#[tokio::test]
async fn test_list_agents() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/agents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    let agents = json["data"]["agents"].as_array().unwrap();
    // Builtin providers have agents
    assert!(!agents.is_empty());
}

#[tokio::test]
async fn test_bind_agent_success() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Create a plan first
    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "Agent Bind Plan",
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

    // Bind an agent (claude-code is supported by builtin providers)
    let bind_payload = serde_json::json!({
        "autoConfig": false
    });

    let bind_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/plans/{}/agents/claude-code/bind", plan_id))
                .header("content-type", "application/json")
                .body(Body::from(bind_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(bind_resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(bind_resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["planId"], plan_id);
    assert_eq!(json["data"]["agentId"], "claude-code");
    assert!(json["data"]["bound"].as_bool().unwrap());
}

#[tokio::test]
async fn test_bind_agent_plan_not_found() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let bind_payload = serde_json::json!({
        "autoConfig": false
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/plans/nonexistent/agents/claude-code/bind")
                .header("content-type", "application/json")
                .body(Body::from(bind_payload.to_string()))
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
async fn test_unbind_agent_not_bound() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Create a plan first
    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "Unbind Test Plan",
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

    // Try to unbind an agent that was never bound
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/plans/{}/agents/claude-code/unbind", plan_id))
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
async fn test_auto_config_agent_not_bound() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Create a plan first
    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "Auto Config Test Plan",
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

    // Try auto-config on an agent that is not bound
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/plans/{}/agents/claude-code/auto-config", plan_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(!json["success"].as_bool().unwrap());
    assert_eq!(json["error"]["code"], "INTERNAL_ERROR");
}
