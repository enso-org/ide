
use crate::prelude::*;

use enso_frp as frp;
use ensogl_core::display;
use ensogl_text as text;
use ensogl_theme as theme;
use ensogl_core::display::shape::StyleWatchFrp;
use ensogl_core::application::Application;

use crate::list_view::Entry;



// ==============
// === Label ===
// ==============

/// A single label entry.
#[derive(Clone,CloneRef,Debug)]
pub struct Label {
    pub(crate) display_object : display::object::Instance,
    pub(crate) label          : text::Area,
    pub(crate) network        : enso_frp::Network,
    pub(crate) style_watch    : StyleWatchFrp,
}

impl Entry for Label {
    type Model = String;

    fn new(app: &Application) -> Self {
        let logger         = Logger::new("list_view::entry::Label");
        let display_object = display::object::Instance::new(logger);
        let label          = app.new_view::<ensogl_text::Area>();
        let network        = frp::Network::new("list_view::entry::Label");
        let style_watch    = StyleWatchFrp::new(&app.display.scene().style_sheet);
        let color          = style_watch.get_color(theme::widget::list_view::text);
        let size           = style_watch.get_number(theme::widget::list_view::text::size);

        display_object.add_child(&label);
        frp::extend! { network
            init  <- source::<()>();
            color <- all(&color,&init)._0();
            size  <- all(&size,&init)._0();

            label.set_default_color     <+ color;
            label.set_default_text_size <+ size.map(|v| text::Size(*v));
            eval size ((t) label.set_position_y(t/2.0));
        }
        init.emit(());
        Self {display_object,label,network,style_watch}
    }

    fn set_model(&self, model: &Self::Model) {
        self.label.set_content(model);
    }

    fn set_label_layer(&self, label_layer:&display::scene::Layer) {
        self.label.add_to_scene_layer(label_layer);
    }
}

impl display::Object for Label {
    fn display_object(&self) -> &display::object::Instance { &self.display_object }
}
