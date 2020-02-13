//! This module contains the base definition of UiComponent.

use nalgebra::Vector2;



// ===============
// === Padding ===
// ===============

/// A struct containing the padding values.
#[derive(Clone,Copy,Debug,Default)]
pub struct Padding {
    /// Left padding.
    pub left   : f32,

    /// Top padding.
    pub top    : f32,

    /// Right padding.
    pub right  : f32,

    /// Bottom padding;
    pub bottom : f32
}



// ===================
// === UiComponent ===
// ===================

/// A trait defining common interfaces of UI components.
pub trait UiComponent {
    /// Sets padding.
    fn set_padding(&mut self, padding:Padding);

    /// Gets padding.
    fn padding(&self) -> Padding;

    /// Sets dimensions.
    fn set_dimensions(&mut self, dimensions:Vector2<f32>);

    /// Gets dimensions.
    fn dimensions(&self) -> Vector2<f32>;

    /// Sets position.
    fn set_position(&mut self, position:Vector2<f32>);

    /// Gets position.
    fn position(&self) -> Vector2<f32>;
}
