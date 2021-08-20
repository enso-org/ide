
use crate::prelude::*;

use enso_frp as frp;
use ensogl_text as text;
use ensogl_theme as theme;
use ensogl_core::display;
use ensogl_core::application::Application;

use crate::list_view::Entry;
use crate::list_view::entry::Label;



// ==================================
// === GlyphHighlightedLabelModel ===
// ==================================

/// The model for [`GlyphHighlightedLabel`], a single label entry with selected glyphs highlighted.
#[allow(missing_docs)]
#[derive(Clone,Debug,Default)]
pub struct GlyphHighlightedLabelModel {
    pub label       : String,
    pub highlighted : Vec<text::Range<text::Bytes>>,
}



// =============================
// === GlyphHighlightedLabel ===
// =============================

/// A single label entry with selected glyphs highlighted.
#[derive(Clone,CloneRef,Debug)]
pub struct GlyphHighlightedLabel {
    inner     : Label,
    highlight : frp::Source<Vec<text::Range<text::Bytes>>>,
}

impl Entry for GlyphHighlightedLabel {
    type Model = GlyphHighlightedLabelModel;

    fn new(app: &Application) -> Self {
        let inner           = Label::new(app);
        let network         = &inner.network;
        let highlight_color = inner.style_watch.get_color(theme::widget::list_view::text::highlight);
        let label           = &inner.label;

        frp::extend! { network
            highlight         <- source::<Vec<text::Range<text::Bytes>>>();
            highlight_changed <- all(highlight,highlight_color);
            eval highlight_changed ([label]((highlight,color)) {
                for range in highlight {
                   label.set_color_bytes(range,color);
                }
            });
        }
        Self {inner,highlight}
    }

    fn set_model(&self, model: &Self::Model) {
        self.inner.set_model(&model.label);
        self.highlight.emit(&model.highlighted);
    }

    fn set_label_layer(&self, layer:&display::scene::Layer) {
        self.inner.set_label_layer(layer);
    }
}

impl display::Object for GlyphHighlightedLabel {
    fn display_object(&self) -> &display::object::Instance { self.inner.display_object() }
}