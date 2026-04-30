//! agent-gateway API Server (library)
//!
//! This crate provides both a binary (`agw-api`) and a library for building
//! and testing the API server.

pub mod state;
pub mod error;
pub mod types;
pub mod handlers;
pub mod middleware;

pub use state::AppState;
