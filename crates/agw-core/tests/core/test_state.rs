//! GlobalState tests

use agw_core::core::state::GlobalState;

#[tokio::test]
async fn test_global_state_new() {
    let state = GlobalState::new();

    // Initial state should have no active plan
    let active = state.get_active_plan_id().await;
    assert!(active.is_none());

    // Fallback config should be default (enabled=true by default)
    let fallback = state.fallback_config.read().await;
    assert!(fallback.enabled); // Default has enabled=true
    assert_eq!(fallback.max_attempts, 3);
}

#[tokio::test]
async fn test_global_state_set_and_get_active_plan() {
    let state = GlobalState::new();

    // Set active plan
    state.set_active_plan("plan-1".to_string()).await;

    // Get active plan
    let active = state.get_active_plan_id().await;
    assert!(active.is_some());
    assert_eq!(active.unwrap(), "plan-1");

    // Set different plan
    state.set_active_plan("plan-2".to_string()).await;

    let active = state.get_active_plan_id().await;
    assert_eq!(active.unwrap(), "plan-2");
}

#[test]
fn test_global_state_bind_agent() {
    let state = GlobalState::new();

    // Bind agent to plan
    state.bind_agent("agent-1", "plan-1");

    // Get plan for agent
    let plan_id = state.get_plan_for_agent("agent-1");
    assert!(plan_id.is_some());
    assert_eq!(plan_id.unwrap(), "plan-1");

    // Bind multiple agents
    state.bind_agent("agent-2", "plan-2");
    state.bind_agent("agent-3", "plan-1");

    assert_eq!(state.get_plan_for_agent("agent-2").unwrap(), "plan-2");
    assert_eq!(state.get_plan_for_agent("agent-3").unwrap(), "plan-1");
}

#[test]
fn test_global_state_unbind_agent() {
    let state = GlobalState::new();

    // Bind then unbind
    state.bind_agent("agent-1", "plan-1");
    assert!(state.get_plan_for_agent("agent-1").is_some());

    state.unbind_agent("agent-1");
    assert!(state.get_plan_for_agent("agent-1").is_none());

    // Unbind non-existent agent should be safe
    state.unbind_agent("nonexistent");
    assert!(state.get_plan_for_agent("nonexistent").is_none());
}

#[test]
fn test_global_state_update_agent_binding() {
    let state = GlobalState::new();

    // Bind agent
    state.bind_agent("agent-1", "plan-1");
    assert_eq!(state.get_plan_for_agent("agent-1").unwrap(), "plan-1");

    // Re-bind to different plan (update)
    state.bind_agent("agent-1", "plan-2");
    assert_eq!(state.get_plan_for_agent("agent-1").unwrap(), "plan-2");
}

#[test]
fn test_global_state_default() {
    let state = GlobalState::default();

    // Default should behave same as new
    let active = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(state.get_active_plan_id());
    assert!(active.is_none());
}