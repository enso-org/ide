//! This module defines the `HasPosition` trait.

use nalgebra::Vector3;

// ===================
// === HasPosition ===
// ===================

/// A trait used to determine that the item implementing it has a 3D position property.
pub trait HasPosition {
    /// Gets self's position.
    fn position(&self) -> Vector3<f32>;
    /// Sets self's position.
    fn set_position(&mut self, position:Vector3<f32>);
}
