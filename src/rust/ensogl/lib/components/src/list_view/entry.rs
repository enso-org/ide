//! A single entry in [`crate::list_view::ListView`].

pub mod label;
pub mod glyph_highlighted_label;
pub mod provider;

pub use glyph_highlighted_label::GlyphHighlightedLabel;
pub use glyph_highlighted_label::GlyphHighlightedLabelModel;
pub use label::Label;
pub use provider::Provider;

use crate::prelude::*;

use ensogl_core::application::Application;
use ensogl_core::display;



// =================
// === Constants ===
// =================

/// Padding inside entry in pixels.
pub const PADDING:f32 = 14.0;
/// The overall entry's height (including padding).
pub const HEIGHT:f32 = 30.0;



// =============
// === Types ===
// =============

/// Entry id. 0 is the first entry in component.
pub type Id = usize;



// =============
// === Entry ===
// =============

/// An object which can be displayed as an entry in [`crate::ListView`] component.
///
/// The entries should not assume any padding - it will be granted by [`ListView`] itself. The
/// display object position of this component is docked to the middle of left entry's boundary. It
/// differs from usual behaviour of EnsoGl components, but makes the entries alignment much simpler.
///
/// This trait abstracts over model and its updating in order to support re-using shapes and gui
/// components, so they are not deleted and created again. The [`ListView`] component does not
/// create [`Entry`] object for each entry provided, and during scrolling, the instantiated objects
/// will be reused: they position will be changed and they will be updated using [`update`] method.
pub trait Entry: CloneRef + Debug + display::Object + 'static {
    /// The model of this entry. The entry should be a visual representation of the [`Model`].
    /// For example, the entry being just a caption can have [`String`] as its model - the text to
    /// be displayed.
    type Model : Debug + Default;

    /// Constructor.
    fn new(app:&Application) -> Self;

    /// Set new model for this entry.
    fn set_model(&self, model:&Self::Model);

    /// Set the layer of all [`text::Area`] components inside. The [`text::Area`] component is
    /// handled in a special way, and is often in different layer than shapes. See TODO comment
    /// in [`text::Area::add_to_scene_layer`] method.
    fn set_label_layer(&self, label_layer:&display::scene::Layer);
}
