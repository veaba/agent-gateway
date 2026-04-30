//! Integration tests for the health check endpoint.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use agw_api::handlers::create_router;

#[tokio::test]
async fn test_health_check_returns_ok() {
    let (state, _temp) = common::setup_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ok");
    assert!(json["version"].as_str().unwrap().len() > 0);
}
