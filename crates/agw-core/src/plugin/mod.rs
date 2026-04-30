//! 插件系统模块

pub mod engine;
pub mod registry;
pub mod lifecycle;
pub mod installer;
pub mod manifest;

pub use engine::*;
pub use registry::*;
pub use lifecycle::*;
pub use installer::*;
pub use manifest::*;