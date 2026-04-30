//! agw-api integration tests.
//!
//! Verifies that the API server binary can be referenced and that basic
//! health-check related types compile correctly.

use agw_core::model_types::*;
use agw_core::test_utils::*;

#[test]
fn test_api_can_import_core_fixtures() {
    let provider = create_test_provider_template();
    assert_eq!(provider.name, "Test Provider");
}

#[test]
fn test_api_can_import_user_plan_fixture() {
    let plan = create_test_user_plan();
    assert_eq!(plan.name, "My Test Plan");
    assert_eq!(plan.health_status, HealthStatus::Unknown);
}
