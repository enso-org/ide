//! This module contains the base definition of UiComponent.

use nalgebra::Vector2;

/// A trait defining common interfaces of UI components.
pub trait UiComponent {
    /// Sets dimensions.
    fn set_dimensions(&mut self, dimensions:Vector2<f32>);

    /// Gets dimensions.
    fn dimensions(&self) -> Vector2<f32>;

    /// Sets position.
    fn set_position(&mut self, position:Vector2<f32>);

    /// Gets position.
    fn position(&self) -> Vector2<f32>;
}
