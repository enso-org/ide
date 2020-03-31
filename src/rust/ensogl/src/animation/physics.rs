//! This module provides inertia physics, used for spring physics, dense fluid dragging and
//! kinematics.

pub mod inertia;

/// Common types.
pub mod types {
    pub use super::inertia::DynInertiaSimulator;
}
pub use types::*;
