//! This module provides a view for project's name which can be used to edit it.

use crate::prelude::*;

use crate::controller;

use ensogl::display;
use ensogl::display::world::World;
use ensogl::display::object::ObjectOps;
use ensogl::display::shape::text::glyph::font;
use ensogl::display::shape::text::text_field::TextField;
use ensogl::display::shape::text::text_field::TextFieldProperties;
use ensogl::data::color;

use nalgebra::Vector2;

/// The project name's view used for visualizing the project name and renaming it.
#[derive(Debug,Clone,CloneRef)]
pub struct ProjectName {
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
        let logger                = Logger::sub(&logger,"ProjectName");
        let display_object        = display::object::Instance::new(&logger);
        let font                  = fonts.get_or_load_embedded_font("DejaVuSansMono").unwrap();
        let size                  = Vector2::new(600.0,100.0);
        let base_color            = color::Rgba::new(1.0, 1.0, 1.0, 0.7);
        let text_size             = 16.0;
        let text_field_properties = TextFieldProperties{base_color,font,size,text_size};
        let text_field            = TextField::new(&world,text_field_properties);
        text_field.set_content(&project_controller.project_name());
        let project_controller = project_controller.clone();
        display_object.add_child(&text_field.display_object());
        Self {logger,display_object,text_field,project_controller}.initialize()
    }

    fn initialize(self) -> Self {
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
        self.setup_center_alignment();
        self
    }

    fn setup_center_alignment(&self) {
        let mut offset = Vector3::new(0.0,0.0,0.0);
        self.text_field.with_mut_content(|content| {
            let mut line = content.line(0);
            offset.x = -line.get_char_x_position(line.len() - 1) / 2.0;
        });
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
