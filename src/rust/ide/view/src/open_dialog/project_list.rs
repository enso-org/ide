use crate::prelude::*;

use enso_frp as frp;
use ensogl_gui_components::list_view;
use ensogl::application::Application;
use ensogl::display;
use ensogl::display::shape::*;
use ensogl_text as text;
use ensogl_gui_components::file_browser;
use ensogl_gui_components::shadow;
use ensogl_theme as theme;
use ensogl::data::color;


pub const WIDTH:f32           = 202.0;
pub const PADDING:f32         = 16.0;
pub const HEIGHT:f32          = file_browser::HEIGHT;
pub const BAR_HEIGHT:f32      = file_browser::TOOLBAR_HEIGHT;
pub const BAR_BORDER_SIZE:f32 = file_browser::TOOLBAR_BORDER_SIZE;
pub const LABEL_SIZE:f32      = list_view::entry::LABEL_SIZE;


#[derive(Clone,CloneRef,Debug)]
pub struct Entry {
    network     : frp::Network,
    style_watch : StyleWatchFrp,
    label       : ensogl_text::Area,
}

impl Entry {
    pub fn new(app:&Application, name:impl Str) -> Self {
        let network     = frp::Network::new("ProjectEntry");
        let label       = app.new_view::<ensogl_text::Area>();
        let style_watch = StyleWatchFrp::new(&app.display.scene().style_sheet);
        let text_color  = style_watch.get_color(ensogl_theme::widget::list_view::text);
        label.set_default_color(text_color.value());
        label.set_position_xy(Vector2(6.0,6.0)); // TODO[ao] Hmmm...
        label.set_content(name.as_ref());
        label.remove_from_scene_layer(&app.display.scene().layers.main);
        label.add_to_scene_layer(&app.display.scene().layers.panel_text);
        frp::extend! { network
            eval text_color ((color) label.set_default_color(color));
        }
        Self {network,style_watch,label}
    }
}

impl display::Object for Entry {
    fn display_object(&self) -> &display::object::Instance { self.label.display_object() }
}

impl list_view::entry::Entry for Entry {
    fn set_focused(&self, _selected: bool) {}

    fn set_width(&self, _width: f32) {}
}

mod background {
    use super::*;

    pub const SHADOW_PX:f32 = 10.0;
    pub const CORNER_RADIUS_PX:f32 = 16.0;

    ensogl::define_shape_system! {
        (style:Style) {
            let sprite_width  : Var<Pixels> = "input_size.x".into();
            let sprite_height : Var<Pixels> = "input_size.y".into();
            let width         = sprite_width - SHADOW_PX.px() * 2.0;
            let height        = sprite_height - SHADOW_PX.px() * 2.0;
            let color         = style.get_color(theme::application::file_browser::background);
            let rect          = Rect((&width,&height)).corners_radius(CORNER_RADIUS_PX.px());
            let shape         = rect.fill(color);

            let shadow  = shadow::from_shape(rect.into(),style);

            let toolbar_border = Rect((width, BAR_BORDER_SIZE.px()))
                .translate_y(height / 2.0 - BAR_HEIGHT.px())
                .fill(style.get_color(theme::application::file_browser::toolbar_border_color));

            (shadow + shape + toolbar_border).into()
        }
    }
}


#[derive(Clone,CloneRef,Debug)]
pub struct ProjectList {
    logger         : Logger,
    display_object : display::object::Instance,
    background     : background::View,
    caption        : text::Area,
    list           : list_view::ListView,
}

impl Deref for ProjectList {
    type Target = list_view::Frp;

    fn deref(&self) -> &Self::Target { &self.list.frp }
}

impl ProjectList {
    pub fn new(app:&Application) -> Self {
        let logger         = Logger::new("ProjectList");
        let display_object = display::object::Instance::new(&logger);
        let background     = background::View::new(&logger);
        let caption        = app.new_view::<text::Area>();
        let list           = app.new_view::<list_view::ListView>();
        display_object.add_child(&background);
        display_object.add_child(&caption);
        display_object.add_child(&list);
        app.display.scene().layers.panel.add_exclusive(&display_object);

        background.size.set(Vector2(WIDTH + background::SHADOW_PX * 2.0,HEIGHT + background::SHADOW_PX * 2.0));

        list.resize(Vector2(WIDTH,HEIGHT-BAR_HEIGHT));
        list.set_position_y(-BAR_HEIGHT/2.0);

        caption.set_position_xy(Vector2(-WIDTH/2.0 + PADDING, HEIGHT/2.0 - PADDING));
        caption.set_default_color(color::Rgba(0.439,0.439,0.439,1.0));
        caption.set_default_text_size(text::Size(LABEL_SIZE));
        caption.set_content("Open Project");
        caption.remove_from_scene_layer(&app.display.scene().layers.main);
        caption.add_to_scene_layer(&app.display.scene().layers.panel_text);

        //TODO[ao] update style.

        Self {logger,display_object,background,caption,list}
    }
}

impl display::Object for ProjectList {
    fn display_object(&self) -> &display::object::Instance { &self.display_object }
}
