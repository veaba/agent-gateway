//! Integration tests for stats endpoints.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use agw_api::handlers::create_router;

#[tokio::test]
async fn test_get_global_stats_empty() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["totalRequests"], 0);
    assert_eq!(json["data"]["totalErrors"], 0);
    assert_eq!(json["data"]["successRate"], 1.0);
    assert_eq!(json["data"]["avgLatencyMs"], 0.0);
    assert_eq!(json["data"]["plansCount"], 0);
    assert!(json["data"]["providersCount"].as_u64().unwrap() > 0);
    assert_eq!(json["data"]["activeAgents"], 0);
}

#[tokio::test]
async fn test_get_provider_stats() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/stats/providers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    let stats = json["data"].as_array().unwrap();
    assert!(!stats.is_empty());
    // Each provider should have stats
    for provider in stats {
        assert!(provider["providerId"].as_str().unwrap().len() > 0);
        assert!(provider["providerName"].as_str().unwrap().len() > 0);
        assert_eq!(provider["totalRequests"], 0);
        assert_eq!(provider["successRate"], 1.0);
    }
}

#[tokio::test]
async fn test_get_plan_stats_not_found() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/stats/nonexistent-plan")
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
async fn test_get_plan_stats_success() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Create a plan first
    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "Stats Plan",
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

    // Get plan stats
    let stats_resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/stats/{}", plan_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(stats_resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(stats_resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["planId"], plan_id);
    assert_eq!(json["data"]["planName"], "Stats Plan");
    assert_eq!(json["data"]["totalRequests"], 0);
    assert_eq!(json["data"]["successRate"], 1.0);
    assert!(json["data"]["quotaUsage"].is_object());
}

#[tokio::test]
async fn test_get_usage_trend_empty() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/stats/usage")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    let points = json["data"]["points"].as_array().unwrap();
    assert!(points.is_empty());
    assert_eq!(json["data"]["granularity"], "hour");
}

#[tokio::test]
async fn test_get_plan_health_not_found() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health/nonexistent-plan")
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
async fn test_get_plan_health_success() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    // Create a plan first
    let payload = serde_json::json!({
        "providerId": "alaya",
        "planId": "alaya-lite",
        "name": "Health Plan",
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

    // Get plan health
    let health_resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/health/{}", plan_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(health_resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(health_resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["planId"], plan_id);
    assert!(json["data"]["status"].as_str().unwrap().len() > 0);
    assert!(json["data"]["checkedAt"].as_str().unwrap().len() > 0);
}
