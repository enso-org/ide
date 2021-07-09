use crate::prelude::*;

use enso_frp as frp;
use ensogl_gui_components::list_view;
use ensogl::application::Application;
use ensogl::display;
use ensogl::display::shape::*;
use ensogl_text as text;
use ensogl_gui_components::shadow;
use ensogl_theme::application::project_list as theme;




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
        let text_color  = style_watch.get_color(theme::text);
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
            let color         = style.get_color(theme::background);
            let border_size   = style.get_number(theme::bar::border_size);
            let bar_height    = style.get_number(theme::bar::height);
            let rect          = Rect((&width,&height)).corners_radius(CORNER_RADIUS_PX.px());
            let shape         = rect.fill(color);

            let shadow  = shadow::from_shape(rect.into(),style);

            let toolbar_border = Rect((width, border_size.px()))
                .translate_y(height / 2.0 - bar_height.px())
                .fill(style.get_color(theme::bar::border_color));

            (shadow + shape + toolbar_border).into()
        }
    }
}


#[derive(Clone,CloneRef,Debug)]
pub struct ProjectList {
    logger         : Logger,
    network        : frp::Network,
    display_object : display::object::Instance,
    background     : background::View,
    caption        : text::Area,
    list           : list_view::ListView,
    style_watch    : StyleWatchFrp,
}

impl Deref for ProjectList {
    type Target = list_view::Frp;

    fn deref(&self) -> &Self::Target { &self.list.frp }
}

impl ProjectList {
    pub fn new(app:&Application) -> Self {
        let logger         = Logger::new("ProjectList");
        let network        = frp::Network::new("ProjectList");
        let display_object = display::object::Instance::new(&logger);
        let background     = background::View::new(&logger);
        let caption        = app.new_view::<text::Area>();
        let list           = app.new_view::<list_view::ListView>();
        display_object.add_child(&background);
        display_object.add_child(&caption);
        display_object.add_child(&list);
        app.display.scene().layers.panel.add_exclusive(&display_object);
        caption.set_content("Open Project");
        caption.remove_from_scene_layer(&app.display.scene().layers.main);
        caption.add_to_scene_layer(&app.display.scene().layers.panel_text);

        let style_watch = StyleWatchFrp::new(&app.display.scene().style_sheet);
        let width       = style_watch.get_number(theme::width);
        let height      = style_watch.get_number(theme::height);
        let bar_height  = style_watch.get_number(theme::bar::height);
        let padding     = style_watch.get_number(theme::padding);
        let color       = style_watch.get_color(theme::bar::label::color);
        let label_size  = style_watch.get_number(theme::bar::label::color);

        frp::extend! { network
            init <- source::<()>();
            size <- all_with3(&width,&height,&init,|w,h,()|
                Vector2(w + background::SHADOW_PX * 2.0,h + background::SHADOW_PX * 2.0)
            );
            list_size  <- all_with4(&width,&height,&bar_height,&init,|w,h,bh,()|
                Vector2(*w,*h - *bh));
            list_y <- bar_height.map(|bh| -*bh / 2.0);
            caption_xy <- all_with4(&width,&height,&padding,&init,
                |w,h,p,()| Vector2(-*w / 2.0 + *p, *h / 2.0 - p)
            );
            color      <- all(&color,&init)._0();
            label_size <- all(&label_size,&init)._0();

            eval size       ((size)  background.size.set(*size));
            eval list_size  ((size)  list.resize(*size));
            eval list_y     ((y)     list.set_position_y(*y));
            eval caption_xy ((xy)    caption.set_position_xy(*xy));
            eval color      ((color) caption.set_default_color(color));
            eval label_size ((size)  caption.set_default_text_size(text::Size(*size)));
        };

        Self {logger,network,display_object,background,caption,list,style_watch}
    }
}

impl display::Object for ProjectList {
    fn display_object(&self) -> &display::object::Instance { &self.display_object }
}
