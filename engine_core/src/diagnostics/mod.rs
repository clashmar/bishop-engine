// engine_core/src/diagnostics/mod.rs
//! Diagnostics infrastructure for engine metrics and performance monitoring.

pub mod metrics;
pub mod collector;

pub use metrics::*;
pub use collector::*;
