//! ListView EnsoGL Component.
//!
//! ListView a displayed list of entries with possibility of selecting one and "choosing" by
//! clicking or pressing enter - similar to the HTML `<select>`.
// #![allow(unused_imports)]
#![allow(unused_qualifications)]

mod column;
pub mod model;
pub mod icons;

use crate::prelude::*;

use model::*;
use crate::shadow;

use enso_frp as frp;
use ensogl_core::application;
use ensogl_core::application::Application;
use ensogl_core::application::shortcut;
use ensogl_core::display;
use ensogl_core::display::shape::*;
use std::path::PathBuf;
use crate::file_browser::column::Column;
use ensogl_core::display::object::ObjectOps;
use ensogl_theme as theme;
use crate::scroll_area::ScrollArea;
use ensogl_text as text;
use ensogl_core::data::color;

const TOOLBAR_HEIGHT      : f32 = 45.0;
const TOOLBAR_BORDER_SIZE : f32 = 1.0;
const CONTENT_OFFSET_Y    : f32 = TOOLBAR_HEIGHT + TOOLBAR_BORDER_SIZE;
const PADDING             : f32 = 16.0;
const WIDTH               : f32 = 814.0;
const HEIGHT              : f32 = 421.0;
const CONTENT_HEIGHT      : f32 = HEIGHT - CONTENT_OFFSET_Y;


// === Background ===

mod background {
    use super::*;

    pub const SHADOW_PX:f32 = 10.0;
    pub const CORNER_RADIUS_PX:f32 = 16.0;

    ensogl_core::define_shape_system! {
        (style:Style) {
            let sprite_width  : Var<Pixels> = "input_size.x".into();
            let sprite_height : Var<Pixels> = "input_size.y".into();
            let width         = sprite_width - SHADOW_PX.px() * 2.0;
            let height        = sprite_height - SHADOW_PX.px() * 2.0;
            let color         = style.get_color(theme::application::file_browser::background);
            let rect          = Rect((&width,&height)).corners_radius(CORNER_RADIUS_PX.px());
            let shape         = rect.fill(color);

            let shadow  = shadow::from_shape(rect.into(),style);

            let toolbar_border = Rect((width, TOOLBAR_BORDER_SIZE.px()))
                .translate_y(height / 2.0 - TOOLBAR_HEIGHT.px())
                .fill(style.get_color(theme::application::file_browser::toolbar_border_color));

            (shadow + shape + toolbar_border).into()
        }
    }
}


// =============
// === Model ===
// =============

/// The Model of Select Component.
#[derive(Clone,Debug)]
struct Model {
    app            : Application,
    display_object : display::object::Instance,
    columns        : RefCell<Vec<Column>>,
    scroll_area    : ScrollArea,
    background     : background::View,
    label          : text::Area,
    focused_column : Cell<usize>,
}

impl Model {
    fn focus_column(&self, column_index:usize) {
        let old_index = self.focused_column.get();
        self.focused_column.set(column_index);
        let columns = self.columns.borrow();
        if let Some(old) = columns.get(old_index) {
            old.model.list_view.defocus();
        }
        if let Some(new) = columns.get(column_index) {
            new.model.list_view.focus();
        }
    }
}



// ===========
// === FRP ===
// ===========

ensogl_core::define_endpoints! {
    Input {
        set_column_width (f32),
        set_content      (AnyFolderContent),
        move_focus_left  (),
        move_focus_right (),
        move_focus_by    (i32),
    }

    Output {
        entry_selected (PathBuf),
        entry_chosen   (PathBuf),
    }
}



// ==============================
// === File Browser Component ===
// ==============================

/// Select Component.
///
/// Select is a displayed list of entries with possibility of selecting one and "chosing" by
/// clicking or pressing enter.
#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug)]
pub struct ModelWithFrp {
    model: Rc<Model>,
    frp: Frp,
}

impl ModelWithFrp {
    /// Constructor.
    pub fn new(app:&Application) -> Rc<Self> {
        let frp     = Frp::new();

        let logger = Logger::new("FileBrowser");
        let display_object = display::object::Instance::new(&logger);

        let background = background::View::new(&logger);
        display_object.add_child(&background);
        app.display.scene().layers.below_main.add_exclusive(&background);
        background.size.set(Vector2(WIDTH+background::SHADOW_PX*2.0, HEIGHT+background::SHADOW_PX*2.0));

        let label = text::Area::new(app);
        display_object.add_child(&label);
        label.add_to_scene_layer(&app.display.scene().layers.label);
        label.set_position_xy(Vector2(-WIDTH/2.0+PADDING, HEIGHT/2.0-PADDING));
        label.set_default_color(color::Rgba(0.439,0.439,0.439,1.0));
        label.set_content("Read files");

        let scroll_area = ScrollArea::new(app);
        display_object.add_child(&scroll_area);
        scroll_area.resize(Vector2(WIDTH,CONTENT_HEIGHT));
        scroll_area.set_position_xy(Vector2(-WIDTH/2.0, HEIGHT/2.0 - CONTENT_OFFSET_Y));
        scroll_area.set_content_width(WIDTH*1.5);

        let focused_column = Cell::new(0);

        let model = Rc::new(Model {
            app: app.clone(),
            display_object,
            columns: RefCell::new(vec![]),
            scroll_area,
            background,
            label,
            focused_column,
        });

        let browser = Rc::new(ModelWithFrp {model,frp});
        let weak_browser = Rc::downgrade(&browser);

        let frp              = &browser.frp;
        let network          = &frp.network;
        let model            = &browser.model;

        frp::extend!{ network
            eval frp.set_content([weak_browser](content) weak_browser.upgrade().unwrap().set_content(content.clone()));

            frp.move_focus_by <+ frp.move_focus_left.constant(-1);
            frp.move_focus_by <+ frp.move_focus_right.constant(1);
            eval frp.move_focus_by([model](amount) {
                let old_index = model.focused_column.get() as i32;
                let new_index = (old_index + amount).max(0).min(model.columns.borrow().len() as i32 - 1);
                model.focus_column(new_index as usize);
            });
        }

        browser
    }

    fn set_content(self:Rc<Self>, content: AnyFolderContent) {
        self.close_columns_from(0);
        self.push_column(content);
        // self.roots_column.set_file_system(fs);
    }

    fn close_columns_from(&self, index:usize) {
        self.model.columns.borrow_mut().truncate(index);
        self.model.scroll_area.set_content_width(self.model.columns.borrow().len() as f32 * column::WIDTH)
    }

    fn push_column(self:Rc<Self>, content:AnyFolderContent) {
        let index   = self.model.columns.borrow().len();
        let network = &self.frp.network;
        frp::extend! { network
            error <- any_mut::<ImString>();
        }
        let new_column = Column::new(self.clone(),index);
        self.model.scroll_area.content.add_child(&new_column);
        self.model.columns.borrow_mut().push(new_column.clone());
        self.model.scroll_area.set_content_width(self.model.columns.borrow().len() as f32 * column::WIDTH);
        self.model.scroll_area.scroll_to_x(f32::INFINITY);
        content.request_entries(new_column.set_entries.clone(),error);
    }
}


// === FileBrowser ===

#[derive(Debug)]
pub struct FileBrowser(Rc<ModelWithFrp>);

impl Deref for FileBrowser {
    type Target = Frp;
    fn deref(&self) -> &Self::Target { &self.0.frp }
}

impl display::Object for FileBrowser {
    fn display_object(&self) -> &display::object::Instance {
        &self.0.model.display_object
    }
}

impl application::command::FrpNetworkProvider for FileBrowser {
    fn network(&self) -> &frp::Network {
        &self.0.frp.network
    }
}

impl application::View for FileBrowser {
    fn label() -> &'static str { "FileBrowser" }
    fn new(app:&Application) -> Self { Self(ModelWithFrp::new(app)) }
    fn app(&self) -> &Application { &self.0.model.app }
    fn default_shortcuts() -> Vec<shortcut::Shortcut> {
        use shortcut::ActionType::*;
        (&[ (PressAndRepeat , "left"  , "move_focus_left")
          , (PressAndRepeat , "right" , "move_focus_right")
          ]).iter().map(|(a,b,c)|Self::self_shortcut(*a,*b,*c)).collect()
    }
}
