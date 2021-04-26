//! UI components for selecting numbers, and a range of numbers.
mod range;
mod number;
mod common;

use ensogl_core::application;
use ensogl_core::application::Application;



// ====================
// === Number Picker ===
// ====================

/// UI component for selecting a number.
pub type NumberPicker = crate::component::Component<common::Model,number::Frp>;

impl application::View for NumberPicker {
    fn label() -> &'static str { "NumberPicker" }
    fn new(app:&Application) -> Self { NumberPicker::new(app) }
    fn app(&self) -> &Application { &self.app }
}



// =========================
// === Number Range Picker ===
// =========================

/// UI component for selecting a number.
pub type NumberRangePicker = crate::component::Component<common::Model,range::Frp>;

impl application::View for NumberRangePicker {
    fn label() -> &'static str { "RangePicker" }
    fn new(app:&Application) -> Self { NumberRangePicker::new(app) }
    fn app(&self) -> &Application { &self.app }
}
