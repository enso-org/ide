//! This module provides the [`FileBrowser`] component. It allows to browse the content of a folder
//! and it's subfolders and emits an event when an entry is chosen.

mod column;
pub mod model;
pub mod icons;

use crate::prelude::*;

use model::*;

use crate::shadow;
use crate::file_browser::column::Column;
use crate::scroll_area::ScrollArea;
use crate::selector::Bounds;
use crate::list_view;

use enso_frp as frp;
use ensogl_core::application::Application;
use ensogl_core::application::shortcut;
use ensogl_core::application;
use ensogl_core::display::object::ObjectOps;
use ensogl_core::display::shape::*;
use ensogl_core::display;
use ensogl_text as text;
use ensogl_theme::application::file_browser as theme;
use std::path::PathBuf;



// =================
// === Constants ===
// =================

// TODO: Take this value from styles. (https://github.com/enso-org/ide/issues/1694)
const TOOLBAR_HEIGHT      : f32 = 45.0;
// TODO: Take this value from styles. (https://github.com/enso-org/ide/issues/1694)
const TOOLBAR_BORDER_SIZE : f32 = 1.0;
// TODO: Take this value from styles. (https://github.com/enso-org/ide/issues/1694)
const CONTENT_OFFSET_Y    : f32 = TOOLBAR_HEIGHT + TOOLBAR_BORDER_SIZE;
// TODO: Take this value from styles. (https://github.com/enso-org/ide/issues/1694)
//       Or make it configurable through FRP, or both.
const WIDTH               : f32 = 814.0;
// TODO: Take this value from styles. (https://github.com/enso-org/ide/issues/1694)
//       Or make it configurable through FRP, or both.
const HEIGHT              : f32 = 421.0;
// TODO: Take this value from styles. (https://github.com/enso-org/ide/issues/1694)
const CONTENT_HEIGHT      : f32 = HEIGHT - CONTENT_OFFSET_Y;



// ==================
// === Background ===
// ==================

mod background {
    use super::*;

    pub const SHADOW_PX        : f32 = 10.0;

    // This defines the background of the file manager, as it can be seen here:
    // https://user-images.githubusercontent.com/1623053/121859507-1028d800-ccf8-11eb-9037-ab4deb835583.png
    // The background consists of the big, almost white rectangle with rounded corners, a drop
    // shadow around the rectangle and the border separating the toolbar at the top.
    ensogl_core::define_shape_system! {
        (style:Style) {
            let sprite_width  : Var<Pixels> = "input_size.x".into();
            let sprite_height : Var<Pixels> = "input_size.y".into();
            let width         = sprite_width - SHADOW_PX.px() * 2.0;
            let height        = sprite_height - SHADOW_PX.px() * 2.0;
            let corner_radius = style.get_number(theme::background::corner_radius);
            let rect          = Rect((&width,&height)).corners_radius(corner_radius.px());
            let color         = style.get_color(theme::background);
            let shape         = rect.fill(color);

            let shadow  = shadow::from_shape(rect.into(),style);

            let toolbar_border = Rect((width,style.get_number(theme::toolbar_border_width).px()));
            let toolbar_height = style.get_number(theme::toolbar_height);
            let toolbar_border = toolbar_border.translate_y(height / 2.0 - toolbar_height.px());

            let toolbar_border_color_theme = theme::toolbar_border_color;
            let toolbar_border_color       = style.get_color(toolbar_border_color_theme);
            let toolbar_border             = toolbar_border.fill(toolbar_border_color);

            (shadow + shape + toolbar_border).into()
        }
    }
}



// =============
// === Model ===
// =============

#[derive(Clone,Debug)]
struct Model {
    app            : Application,
    display_object : display::object::Instance,
    columns        : RefCell<Vec<Column>>,
    scroll_area    : ScrollArea,
    background     : background::View,
    title          : text::Area,
    /// The index of the column that is currently focused.
    /// Invariant: The column with this index (if it exists) has `focused` set, all other columns
    ///            have it unset.
    focused_column : Cell<usize>,
}

impl Model {
    fn focus_column(&self, index:usize) {
        let old_index  = self.focused_column.replace(index);
        let old_column = self.columns.borrow().get(old_index).cloned();
        if let Some(old_column) = old_column {
            old_column.defocus();
        }
        let new_column = self.columns.borrow().get(index).cloned();
        if let Some(new_column) = new_column {
            new_column.focus();
            self.scroll_to_column(&new_column);
        }
    }

    fn move_focus_by(&self, amount:isize) {
        let old_index  = self.focused_column.get();
        let max_index  = self.columns.borrow().len() as isize - 1;
        let new_index  = old_index as isize + amount;
        let new_index  = new_index.max(0).min(max_index);
        let new_index  = new_index as usize;
        let new_column = self.columns.borrow().get(new_index).cloned();
        if let Some(new_column) = new_column {
            let entries = new_column.model.entries.borrow().as_ref().cloned();
            if let Some(entries) = entries {
                if entries.len() > 0 {
                    self.focus_column(new_index);
                }
            }
        }
    }

    fn scroll_to_column(&self,column:&Column) {
        let styles = StyleWatch::new(&self.app.display.scene().style_sheet);
        let spacing = styles.get_number(theme::scroll_spacing);
        let left  = (column.left.value() - spacing).max(0.0);
        let right = column.right.value() + spacing;
        self.scroll_area.scroll_to_x_range(Bounds::new(left,right));
    }

    fn update_content_width(&self) {
        let styles = StyleWatch::new(&self.app.display.scene().style_sheet);
        let spacing = styles.get_number(theme::scroll_spacing);
        let content_width = if let Some(last_column) = self.columns.borrow().last() {
            last_column.left.value() + WIDTH - spacing
        } else {
            WIDTH
        };
        self.scroll_area.set_content_width(content_width);
    }

    fn close_columns_from(&self, index:usize) {
        self.columns.borrow_mut().truncate(index);
        self.update_content_width();
    }
}



// ===========
// === FRP ===
// ===========

ensogl_core::define_endpoints! {
    Input {
        set_content      (AnyFolderContent),
        move_focus_left  (),
        move_focus_right (),
        move_focus_by    (isize),

        copy_focused       (),
        cut_focused        (),
        paste_into_focused (),
    }

    Output {
        entry_selected (PathBuf),
        entry_chosen   (PathBuf),

        copy       (PathBuf),
        cut        (PathBuf),
        paste_into (PathBuf),
    }
}



// ====================
// === ModelWithFrp ===
// ====================

/// A `Model` with an initialized FRP network. This contains all the actual data and behavior of a
/// file browser.
#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug)]
pub struct ModelWithFrp {
    model : Rc<Model>,
    frp   : Frp,
}

impl ModelWithFrp {

    /// Constructor.
    pub fn new(app:&Application) -> Rc<Self> {
        let app            = app.clone();
        let scene          = app.display.scene();
        let styles         = StyleWatchFrp::new(&scene.style_sheet);
        let logger         = Logger::new("FileBrowser");
        let display_object = display::object::Instance::new(&logger);
        scene.layers.panel.add_exclusive(&display_object);
        let columns        = RefCell::new(vec![]);
        let focused_column = Cell::new(0);


        // === Background ===

        let background = background::View::new(&logger);
        display_object.add_child(&background);
        scene.layers.add_shapes_order_dependency::<background::View,list_view::selection::View>();


        // === Title ===

        let title = text::Area::new(&app);
        display_object.add_child(&title);
        title.add_to_scene_layer(&scene.layers.panel_text);
        title.set_content("Read files");


        // === Scroll Area ===

        let scroll_area = ScrollArea::new(&app);
        display_object.add_child(&scroll_area);


        // === Browser ===

        let model = Rc::new(Model {app,display_object,columns,scroll_area,background,title,
            focused_column});
        let frp          = Frp::new();
        let browser      = Rc::new(ModelWithFrp {model:model.clone(),frp:frp.clone()});
        let weak_browser = Rc::downgrade(&browser);


        // === FRP ===

        let network = &frp.network;
        frp::extend!{ network

            // === Set Content ===

            eval frp.set_content([weak_browser](content)
                if let Some(browser) = weak_browser.upgrade() {
                    browser.set_content(content.clone());
                }
            );


            // === Focus ===

            frp.move_focus_by <+ frp.move_focus_left.constant(-1);
            frp.move_focus_by <+ frp.move_focus_right.constant(1);
            eval frp.move_focus_by((&amount) model.move_focus_by(amount));
            focused_entry <= all_with(&frp.move_focus_by,&frp.entry_selected,f!([model](_,_) {
                Some(model.columns.borrow().get(model.focused_column.get())?.entry_selected.value())
            }));


            // === Copy, Cut and Paste ===

            frp.source.copy       <+ focused_entry.sample(&frp.copy_focused);
            frp.source.cut        <+ focused_entry.sample(&frp.cut_focused);
            frp.source.paste_into <+ focused_entry.sample(&frp.paste_into_focused);


            // === Styles ===

            init <- source::<()>();
            let toolbar_height = styles.get_number(theme::toolbar_height);
            toolbar_height <- all(&toolbar_height,&init)._0();
            let toolbar_border_width = styles.get_number(theme::toolbar_border_width);
            toolbar_border_width <- all(&toolbar_border_width,&init)._0();
            content_offset_y <- all_with(&toolbar_height,&toolbar_border_width,|&toolbar,&border|
                toolbar + border);
            let width = styles.get_number(theme::width);
            width <- all(&width,&init)._0();
            let height = styles.get_number(theme::height);
            height <- all(&height,&init)._0();
            let padding = styles.get_number(theme::padding);
            padding <- all(&padding,&init)._0();
            let title_color = styles.get_color(theme::title_color);
            title_color <- all(&title_color,&init)._0();


            // === Scroll Are Position ===

            scroll_area_x <- width.map(|&width| -width / 2.0);
            eval scroll_area_x((&x) model.scroll_area.set_position_x(x));
            scroll_area_y <- all_with(&height,&content_offset_y,|&height,&offset|
                height / 2.0 - offset);
            eval scroll_area_y((&y) model.scroll_area.set_position_y(y));

            content_height <- all_with(&content_offset_y,&height,|&offset,&height| height - offset);
            content_size <- all_with(&width,&content_height,|&width,&height| Vector2(width,height));
            eval content_size((&size) model.scroll_area.resize(size));


            // === Background ===

            background_size <- all_with(&width,&height,|&width,&height|
                Vector2(width + background::SHADOW_PX * 2.0,height + background::SHADOW_PX * 2.0));
            eval background_size((&size) model.background.size.set(size));


            // === Title ===

            title_position <- all_with3(&width,&height,&padding,|&width,&height,&padding|
                Vector2(-width/2.0+padding,height/2.0-padding));
            eval title_position((&position) model.title.set_position_xy(position));

            eval title_color((&color) model.title.set_color_all(color));
        }

        init.emit(());

        browser
    }

    fn set_content(self:&Rc<Self>, content: AnyFolderContent) {
        self.model.close_columns_from(0);
        self.push_column(content);
        self.model.focus_column(0);
    }

    fn push_column(self:&Rc<Self>, content:AnyFolderContent) {
        let index      = self.model.columns.borrow().len();
        let new_column = Column::new(self.clone(),index);
        self.model.scroll_area.content().add_child(&new_column);
        self.model.columns.borrow_mut().push(new_column.clone());
        self.model.update_content_width();
        self.model.scroll_to_column(&new_column);
        content.request_entries(new_column.set_entries.clone(),new_column.set_error.clone());
    }
}



// ===================
// === FileBrowser ===
// ===================

/// A file browser component. It allows to browse the content of a folder and it's subfolders and
/// emits an event when an entry is chosen.
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
        (&[ (PressAndRepeat , "left"   , "move_focus_left")
          , (PressAndRepeat , "right"  , "move_focus_right")
          , (PressAndRepeat , "ctrl c" , "copy_focused")
          , (PressAndRepeat , "ctrl x" , "cut_focused")
          , (PressAndRepeat , "ctrl v" , "paste_into_focused")
          ]).iter().map(|(a,b,c)|Self::self_shortcut(*a,*b,*c)).collect()
    }
}
