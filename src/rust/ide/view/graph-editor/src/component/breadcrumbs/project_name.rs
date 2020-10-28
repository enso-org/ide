//! This module provides a view for project's name which can be used to edit it.

use crate::prelude::*;

use crate::component::breadcrumbs::TEXT_SIZE;
use crate::component::breadcrumbs::GLYPH_WIDTH;
use crate::component::breadcrumbs::VERTICAL_MARGIN;
use crate::component::breadcrumbs::breadcrumb;

use enso_frp as frp;
use ensogl::application::Application;
use ensogl::data::color;
use ensogl::display::object::ObjectOps;
use ensogl::display::shape::*;
use ensogl::display;
use ensogl::gui::component::Animation;
use ensogl::gui::component;
use ensogl_text as text;
use ensogl_text::style::Size as TextSize;
use ensogl_theme as theme;
use logger::AnyLogger;
use logger::enabled::Logger;



// =================
// === Constants ===
// =================

/// Project name used as a placeholder in `ProjectName` view when it's initialized.
pub const UNKNOWN_PROJECT_NAME:&str = "Unknown";
/// Default line height for project names.
pub const LINE_HEIGHT : f32 = TEXT_SIZE * 1.5;



// ==================
// === Background ===
// ==================

mod background {
    use super::*;

    ensogl::define_shape_system! {
        () {
            let bg_color = color::Rgba::new(0.0,0.0,0.0,0.000_001);
            Plane().fill(bg_color).into()
        }
    }
}



// ===========
// === FRP ===
// ===========

ensogl::define_endpoints! {
    Input {
       /// Set the project name.
       name (String),
       /// Reset the project name to the one before editing.
       cancel_editing (),
       /// Commit current project name.
       commit        (),
       outside_press (),
       select        (),
       deselect      (),

    }
    Output {
        name       (String),
        width      (f32),
        mouse_down (),
        edit_mode  (bool),
        selected   (bool)
    }
}



// ==================
// === Animations ===
// ==================

/// Animation handlers.
#[derive(Debug,Clone,CloneRef)]
pub struct Animations {
    color    : Animation<color::Rgba>,
    position : Animation<Vector3<f32>>
}

impl Animations {
    /// Constructor.
    pub fn new(network:&frp::Network) -> Self {
        let color    = Animation::new(&network);
        let position = Animation::new(&network);
        Self{color,position}
    }
}



// ========================
// === ProjectNameModel ===
// ========================

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
struct ProjectNameModel {
    logger         : Logger,
    display_object : display::object::Instance,
    view           : component::ShapeView<background::Shape>,
    style          : StyleWatch,
    text_field     : text::Area,
    project_name   : Rc<RefCell<String>>,
}

impl ProjectNameModel {
    /// Constructor.
    fn new(app:&Application) -> Self {
        let scene                 = app.display.scene();
        let logger                = Logger::new("ProjectName");
        let display_object        = display::object::Instance::new(&logger);
        // FIXME : StyleWatch is unsuitable here, as it was designed as an internal tool for shape system (#795)
        let style                 = StyleWatch::new(&scene.style_sheet);
        let base_color            = style.get_color(theme::vars::graph_editor::breadcrumbs::transparent::color);
        let base_color            = color::Rgba::from(base_color);
        let text_size:TextSize    = TEXT_SIZE.into();
        let text_field            = app.new_view::<text::Area>();
        text_field.set_default_color.emit(base_color);
        text_field.set_default_text_size(text_size);
        text_field.single_line(true);

        text_field.remove_from_view(&scene.views.main);
        text_field.add_to_view(&scene.views.breadcrumbs);
        text_field.hover();

        let view_logger           = Logger::sub(&logger,"view_logger");
        let view                  = component::ShapeView::<background::Shape>::new(&view_logger, scene);

        scene.views.main.remove_shape_view(&view);
        scene.views.breadcrumbs.add_shape_view(&view);

        let project_name          = Rc::new(RefCell::new(UNKNOWN_PROJECT_NAME.to_string()));
        Self{logger,view,style,display_object,text_field,project_name}.init()
    }

    /// Compute the width of the ProjectName view.
    fn width(&self, content:&str) -> f32 {
        let glyphs = content.len();
        let width  = glyphs as f32 * GLYPH_WIDTH;
        width + breadcrumb::LEFT_MARGIN + breadcrumb::RIGHT_MARGIN + breadcrumb::PADDING * 2.0
    }

    fn update_alignment(&self, content:&str) {
        let width       = self.width(content);
        let line_height = LINE_HEIGHT;
        let height      = line_height+VERTICAL_MARGIN*2.0;
        let x_position  = breadcrumb::LEFT_MARGIN + breadcrumb::PADDING;
        let y_position  = -VERTICAL_MARGIN - breadcrumb::TOP_MARGIN - breadcrumb::PADDING;
        self.text_field.set_position(Vector3(x_position,y_position,0.0));
        self.view.shape.sprite.size.set(Vector2(width,height));
        self.view.set_position(Vector3(width,-height,0.0)/2.0);
    }

    fn init(self) -> Self {
        self.add_child(&self.text_field);
        self.text_field.add_child(&self.view);
        self.update_text_field_content(self.project_name.borrow().as_str());
        self
    }

    fn reset_name(&self) {
        info!(self.logger, "Resetting project name.");
        self.update_text_field_content(self.project_name.borrow().as_str());
    }

    fn update_text_field_content(&self, content:&str) {
        self.text_field.set_content(content);
        self.update_alignment(content);
    }

    fn set_color(&self, value:color::Rgba) {
        self.text_field.set_default_color(value);
        self.text_field.set_color_all(value);
    }

    fn set_position(&self, value:Vector3<f32>) {
        self.text_field.set_position(value);
    }

    fn rename(&self, name:impl Str) {
        let name = name.into();
        self.update_text_field_content(&name);
    }

    fn commit<T:Into<String>>(&self, name:T) {
        let name = name.into();
        debug!(self.logger, "Committing name: '{name}'.");
        *self.project_name.borrow_mut() = name;
    }
}

impl display::Object for ProjectNameModel {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}



// ===================
// === ProjectName ===
// ===================

/// The view used for displaying and renaming it.
#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct ProjectName {
    model   : Rc<ProjectNameModel>,
    pub frp : Frp
}

impl ProjectName {
    /// Constructor.
    pub fn new(app:&Application) -> Self {
        let frp     = Frp::new_network();
        let model   = Rc::new(ProjectNameModel::new(app));
        let network = &frp.network;
        let scene   = app.display.scene();
        let text    = &model.text_field.frp;
        // FIXME : StyleWatch is unsuitable here, as it was designed as an internal tool for shape system (#795)
        let styles            = StyleWatch::new(&scene.style_sheet);
        let hover_color       = styles.get_color(theme::vars::graph_editor::breadcrumbs::hover::color);
        let hover_color       = color::Rgba::from(hover_color);
        let deselected_color  = styles.get_color(theme::vars::graph_editor::breadcrumbs::deselected::left::color);
        let deselected_color  = color::Rgba::from(deselected_color);
        let selected_color    = styles.get_color(theme::vars::graph_editor::breadcrumbs::selected::color);
        let selected_color    = color::Rgba::from(selected_color);

        let animations = Animations::new(&network);

        frp::extend! { network


            // === Mouse IO ===

            not_selected               <- frp.output.selected.map(|selected| !selected);
            mouse_over_if_not_selected <- model.view.events.mouse_over.gate(&not_selected);
            mouse_out_if_not_selected  <- model.view.events.mouse_out.gate(&not_selected);
            eval_ mouse_over_if_not_selected(
                animations.color.set_target_value(hover_color);
            );
            eval_ mouse_out_if_not_selected(
                animations.color.set_target_value(deselected_color);
            );
            on_deselect <- not_selected.gate(&not_selected).constant(());

            frp.output.source.mouse_down <+ model.view.events.mouse_down;
            start_edit_mode <- model.view.events.mouse_down.constant(());
            eval_ start_edit_mode ( text.set_cursor_at_mouse_position() );
            frp.source.edit_mode <+ start_edit_mode.to_true();

            outside_press     <- any(&frp.outside_press,&frp.deselect);


            // === Text Area ===

            text_content <- text.content.map(|txt| txt.to_string());
            eval text_content((content) model.update_alignment(&content));
            text_width <- text_content.map(f!((content) model.width(content)));
            frp.source.width <+ text_width;


            // === Input Commands ===

            eval_ frp.input.cancel_editing(model.reset_name());
            eval  frp.input.name((name) {model.rename(name)});
            frp.output.source.name <+ frp.input.name;


            // === Commit ===

            do_commit <- any(&frp.commit,&outside_press).gate(&frp.output.edit_mode);
            commit_text <- text_content.sample(&do_commit);
            frp.output.source.name <+ commit_text;
            eval commit_text((text) model.commit(text));
            on_commit <- commit_text.constant(());

            end_edit_mode <- any(&on_commit,&on_deselect);
            frp.output.source.edit_mode <+ end_edit_mode.to_false();


            // === Selection ===

            select <- any(&frp.select,&start_edit_mode);
            eval_  select([text,animations]{
                text.set_focus(true);
                animations.color.set_target_value(selected_color);
            });
            frp.output.source.selected <+ select.to_true();

            deselect <- any(&frp.deselect,&end_edit_mode);
            eval_ deselect ([text,animations]{
                text.set_focus(false);
                text.remove_all_cursors();
                animations.color.set_target_value(deselected_color);
            });
            frp.output.source.selected <+ deselect.to_false();


            // === Animations ===

            eval animations.color.value((value) model.set_color(*value));
            eval animations.position.value((value) model.set_position(*value));

        }

        frp.deselect();

        Self{frp,model}
    }

}

impl display::Object for ProjectName {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object
    }
}
