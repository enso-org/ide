//! This module provides a view for project's name which can be used to edit it.

use crate::prelude::*;

use crate::controller;

use enso_frp as frp;
use ensogl::data::color;
use ensogl::display;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::object::ObjectOps;
use ensogl::display::scene::Scene;
use ensogl::display::shape::text::glyph::font;
use ensogl::display::shape::text::text_field::TextField;
use ensogl::display::shape::text::text_field::TextFieldProperties;
use ensogl::display::shape::*;
use ensogl::display::Sprite;
use ensogl::display::world::World;
use ensogl::gui::component::Animation;
use ensogl::gui::component;
use logger::enabled::Logger;
use logger::AnyLogger;
use nalgebra::Vector2;



// =================
// === Constants ===
// =================

const HIGHLIGHTED_TEXT_COLOR : color::Rgba = color::Rgba::new(1.0,1.0,1.0,1.0);
const DARK_GRAY_TEXT_COLOR   : color::Rgba = color::Rgba::new(0.6,0.6,0.6,1.0);



// =============
// === Shape ===
// =============

mod shape {
    use super::*;

    ensogl::define_shape_system! {
            (style:Style, selection:f32) {
                let bg_color = color::Rgba::new(0.0,0.0,0.0,0.000001);
                let width  : Var<Distance<Pixels>> = "input_size.x".into();
                let height : Var<Distance<Pixels>> = "input_size.y".into();
                let shape  = Rect((&width,&height));
                let shape  = shape.fill(bg_color);
                shape.into()
            }
        }
}

/// The project name's view used for visualizing the project name and renaming it.
#[derive(Debug,Clone,CloneRef)]
pub struct ProjectName {
    network            : frp::Network,
    main_area          : component::ShapeView<shape::Shape>,
    logger             : Logger,
    display_object     : display::object::Instance,
    text_field         : TextField,
    project_controller : controller::Project
}

impl ProjectName {
    /// Create a new ProjectName view.
    pub fn new
    ( logger             : impl AnyLogger
    , world              : &World
    , project_controller : &controller::Project
    , fonts              : &mut font::Registry
    ) -> Self {
        let scene                 = world.scene();
        let network               = frp::Network::new();
        let logger                = Logger::sub(&logger,"ProjectName");
        let display_object        = display::object::Instance::new(&logger);
        let font                  = fonts.get_or_load_embedded_font("DejaVuSansMono").unwrap();
        let size                  = Vector2::new(600.0,100.0);
        let base_color            = DARK_GRAY_TEXT_COLOR;
        let text_size             = 16.0;
        let text_field_properties = TextFieldProperties{base_color,font,size:size.clone(),text_size};
        let text_field            = TextField::new(&world,text_field_properties);
        let project_controller    = project_controller.clone();
        let main_logger           = Logger::sub(&logger,"main_area");
        let main_area             = component::ShapeView::<shape::Shape>::new(&main_logger,scene);
        Self{main_area,network,logger,display_object,text_field,project_controller}.initialize()
    }

    fn initialize(self) -> Self {
        self.text_field.set_content(&self.project_controller.project_name());
        self.display_object.add_child(&self.text_field.display_object());
        self.display_object.add_child(&self.main_area);
        let project_name = self.clone_ref();
        self.text_field.set_text_edit_callback(move |change| {
            // If the text edit callback is called, the TextEdit must be still alive.
            let field_content = project_name.text_field.get_content();
            let new_name      = field_content.replace("\n","");
            if change.inserted == "\n" {
                project_name.rename(&new_name);
            }
            // Keep only one line.
            project_name.text_field.set_content(&new_name);
            project_name.setup_center_alignment();
        });

        let network      = &self.network;
        let project_name = self.clone_ref();
        frp::extend! { network
            def _foo = self.main_area.events.mouse_over.map(f_!(project_name.highlight_text()));
            def _foo = self.main_area.events.mouse_out .map(f_!(project_name.darken_text()));
        }

        self.setup_center_alignment();
        self
    }

    fn defocus(&self) {
        self.darken_text();
    }

    fn highlight_text(&self) {
        self.text_field.set_base_color(HIGHLIGHTED_TEXT_COLOR)
    }

    fn darken_text(&self) {
        if !self.text_field.is_focused() {
            self.text_field.set_base_color(DARK_GRAY_TEXT_COLOR);
        }
    }

    fn setup_center_alignment(&self) {
        let mut width = 0.0;
        self.text_field.with_mut_content(|content| {
            let mut line = content.line(0);
            if line.len() > 0 {
                width = line.get_char_x_position(line.len() - 1);
            }
        });
        let offset = Vector3::new(-width/2.0,0.0,0.0);
        let height = 16.0;
        self.main_area.shape.sprite.size.set(Vector2::new(width,height));
        self.main_area.set_position(Vector3::new(0.0,-height/2.0,0.0));
        self.text_field.set_position(offset);
    }

    /// Get the project name.
    pub fn name(&self) -> String {
        self.project_controller.project_name().to_string()
    }

    /// Change the project name.
    pub fn rename(&self, new_name:impl Str) {
        let new_name = new_name.into();
        let old_name = self.name();
        if new_name != old_name {
            info!(self.logger, "Renaming '{old_name}' to '{new_name}'");
            let project_name = self.clone_ref();
            executor::global::spawn(async move {
                if let Err(e) = project_name.project_controller.rename_project(&new_name).await {
                    info!(project_name.logger, "Couldn't rename project to '{new_name}': {e}");
                    project_name.text_field.set_content(&old_name);
                    project_name.setup_center_alignment();
                }
            });
        }
    }
}

impl display::Object for ProjectName {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
