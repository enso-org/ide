//! Base model for the number and range selector components.
use crate::prelude::*;

use crate::component;
use crate::selector::common;

use enso_frp as frp;
use ensogl_core::application::Application;
use ensogl_core::application;
use ensogl_core::display::shape::*;
use ensogl_core::display;
use ensogl_text as text;

use super::shape::*;

const LABEL_OFFSET : f32 = 13.0;


// ==============================================
// === Utilities - Decimal Aligned Text Field ===
// ==============================================

mod decimal_aligned {
    use super::*;

    ensogl_core::define_endpoints! {
        Input {
            set_content(f32),
        }
        Output {}
    }

    impl component::Frp<Model> for Frp {
        fn init(&self, app: &Application, model: &Model, _style: &StyleWatchFrp) {
            let frp     = &self;
            let network = &frp.network;
            let _scene   = app.display.scene();

            frp::extend! { network
                formatted <- frp.set_content.map(|value| format!("{:.2}", value));
                // FIXME: the next line is locale dependent. We need a way to get the current locale
                //  dependent decimal separator for this.
                left      <- formatted.map(|s| s.split('.').next().map(|s| s.to_string())).unwrap();

                model.label_left.set_content  <+ left;
                model.label.set_content       <+ formatted;

                eval model.label_left.width((offset)  model.label.set_position_x(-offset-LABEL_OFFSET));
            }
        }
    }

    #[derive(Clone,CloneRef,Debug)]
    pub struct Model {
        root       : display::object::Instance,
        label      : text::Area,
        label_left : text::Area,
        label_right: text::Area,
    }

    impl component::Model for Model {
        fn new(app:&Application) -> Self {
            let logger             = Logger::new("DecimalAlignedLabel");
            let root               = display::object::Instance::new(&logger);
            let label              = app.new_view::<text::Area>();
            let label_left         = app.new_view::<text::Area>();
            let label_right        = app.new_view::<text::Area>();

            label.remove_from_scene_layer_DEPRECATED(&app.display.scene().layers.main);
            label.add_to_scene_layer_DEPRECATED(&app.display.scene().layers.label);

            root.add_child(&label);
            root.add_child(&label_left);
            root.add_child(&label_right);

            Self{root,label,label_left,label_right}
        }
    }

    impl display::Object for Model {
        fn display_object(&self) -> &display::object::Instance { self.label.display_object() }
    }

    pub type FloatLabel = crate::component::Component<Model,Frp>;

    impl application::View for FloatLabel {
        fn label() -> &'static str { "DecimalAlignedLabel" }
        fn new(app:&Application) -> Self { FloatLabel::new(app) }
        fn app(&self) -> &Application { &self.app }
    }
}



// =============
// === Model ===
// =============

#[derive(Clone,CloneRef,Debug)]
pub struct Model {
    pub background         : background::View,
    pub track              : track::View,
    pub track_handle_left  : io_rect::View,
    pub track_handle_right : io_rect::View,
    pub left_overflow      : left_overflow::View,
    pub right_overflow     : right_overflow::View,
    pub label              : decimal_aligned::FloatLabel,
    pub label_left         : text::Area,
    pub label_right        : text::Area,
    pub caption_left       : text::Area,
    pub caption_center     : text::Area,
    pub root               : display::object::Instance,
}

impl component::Model for Model {
    fn new(app: &Application) -> Self {
        let logger             = Logger::new("selector::common::Model");
        let root               = display::object::Instance::new(&logger);
        let label              = app.new_view::<decimal_aligned::FloatLabel>();
        let label_left         = app.new_view::<text::Area>();
        let label_right        = app.new_view::<text::Area>();
        let caption_center     = app.new_view::<text::Area>();
        let caption_left       = app.new_view::<text::Area>();
        let background         = background::View::new(&logger);
        let track              = track::View::new(&logger);
        let track_handle_left  = io_rect::View::new(&logger);
        let track_handle_right = io_rect::View::new(&logger);
        let left_overflow      = left_overflow::View::new(&logger);
        let right_overflow     = right_overflow::View::new(&logger);

        let app        = app.clone_ref();
        let scene      = app.display.scene();
        scene.layers.add_shapes_order_dependency::<background::View, track::View>();
        scene.layers.add_shapes_order_dependency::<track::View, left_overflow::View>();
        scene.layers.add_shapes_order_dependency::<track::View, right_overflow::View>();
        scene.layers.add_shapes_order_dependency::<track::View, io_rect::View>();

        root.add_child(&label);
        root.add_child(&label_left);
        root.add_child(&label_right);
        root.add_child(&caption_left);
        root.add_child(&caption_center);
        root.add_child(&background);
        root.add_child(&track);
        root.add_child(&right_overflow);

        label_left.remove_from_scene_layer_DEPRECATED(&scene.layers.main);
        label_left.add_to_scene_layer_DEPRECATED(&scene.layers.label);
        label_right.remove_from_scene_layer_DEPRECATED(&scene.layers.main);
        label_right.add_to_scene_layer_DEPRECATED(&scene.layers.label);
        caption_left.remove_from_scene_layer_DEPRECATED(&scene.layers.main);
        caption_left.add_to_scene_layer_DEPRECATED(&scene.layers.label);
        caption_center.remove_from_scene_layer_DEPRECATED(&scene.layers.main);
        caption_center.add_to_scene_layer_DEPRECATED(&scene.layers.label);

        Self{root,label,background,track,left_overflow,right_overflow,caption_left,caption_center,
            label_left,label_right,track_handle_left,track_handle_right}
    }
}

impl Model {
    pub fn set_size(&self, size:Vector2, shadow_padding:Vector2) {
        let padded_size = size+shadow_padding;
        self.background.size.set(padded_size);
        self.track.size.set(padded_size);
        self.left_overflow.size.set(padded_size);
        self.right_overflow.size.set(padded_size);

        let left_padding       = LABEL_OFFSET;
        let overflow_icon_size = size.y;
        let label_offset       = size.x / 2.0 - overflow_icon_size + left_padding;

        self.label_left.set_position_x(-label_offset);
        self.label_right.set_position_x(label_offset-self.label_right.width.value());

        let overflow_icon_offset = size.x / 2.0 - overflow_icon_size / 2.0;
        self.left_overflow.set_position_x(-overflow_icon_offset);
        self.right_overflow.set_position_x(overflow_icon_offset);

        let track_handle_size = Vector2::new(size.y/2.0,size.y);
        self.track_handle_left.size.set(track_handle_size);
        self.track_handle_right.size.set(track_handle_size);
    }

    pub fn update_caption_position(&self, (size,text_size):&(Vector2,f32)) {
        let left_padding = LABEL_OFFSET;
        let overflow_icon_size = size.y / 2.0;
        let caption_offset   = size.x / 2.0 - overflow_icon_size - left_padding;
        self.caption_left.set_position_x(-caption_offset);
        self.caption_left.set_position_y(text_size / 2.0);
        self.caption_center.set_position_y(text_size / 2.0);
    }

    pub fn use_track_handles(&self, value:bool) {
        if value {
            self.track.add_child(&self.track_handle_left);
            self.track.add_child(&self.track_handle_right);
        } else {
            self.track.remove_child(&self.track_handle_left);
            self.track.remove_child(&self.track_handle_right);
        }
    }

    pub fn set_background_value(&self, value:f32) {
        self.track.left.set(0.0);
        self.track.right.set(value);
    }

    pub fn set_background_range(&self, value:common::Range, size:Vector2) {
        self.track.left.set(value.0);
        self.track.right.set(value.1);

        self.track_handle_left.set_position_x(value.0 * size.x - size.x / 2.0);
        self.track_handle_right.set_position_x(value.1 * size.x  - size.x / 2.0);
    }

    pub fn set_center_label_content(&self, value:f32) {
        self.label.frp.set_content.emit(value)
    }

    pub fn set_left_label_content(&self, value:f32) {
        self.label_left.frp.set_content.emit(format!("{:.2}", value))
    }

    pub fn set_right_label_content(&self, value:f32) {
        self.label_right.frp.set_content.emit(format!("{:.2}", value))
    }

    pub fn set_caption_left(&self, caption:Option<String>) {
        let caption = caption.unwrap_or_default();
        self.caption_left.frp.set_content.emit(caption);
    }

    pub fn set_caption_center(&self, caption:Option<String>) {
        let caption = caption.unwrap_or_default();
        self.caption_center.frp.set_content.emit(caption);
    }

    pub fn show_left_overflow(&self, value:bool) {
        if value {
            self.root.add_child(&self.left_overflow);
        } else {
            self.root.remove_child(&self.left_overflow);
        }
    }

    pub fn show_right_overflow(&self, value:bool) {
        if value {
            self.root.add_child(&self.right_overflow);
        } else {
            self.root.remove_child(&self.right_overflow);
        }
    }

    pub fn left_corner_round(&self,value:bool) {
        let corner_roundness = if value { 1.0 } else { 0.0 };
        self.background.corner_left.set(corner_roundness);
        self.track.corner_left.set(corner_roundness)
    }

    pub fn right_corner_round(&self,value:bool) {
        let corner_roundness = if value { 1.0 } else { 0.0 };
        self.background.corner_right.set(corner_roundness);
        self.track.corner_right.set(corner_roundness)
    }
}

impl display::Object for Model {
    fn display_object(&self) -> &display::object::Instance { &self.root }
}
