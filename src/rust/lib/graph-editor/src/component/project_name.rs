//! This module provides a view for project's name which can be used to edit it.

use crate::prelude::*;

use enso_frp as frp;
use ensogl::data::color;
use ensogl::display;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::object::ObjectOps;
use ensogl::display::scene::Scene;
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
use ensogl::animation::linear_interpolation;


// =================
// === Constants ===
// =================

const HIGHLIGHTED_TEXT_COLOR : color::Rgba = color::Rgba::new(1.0,1.0,1.0,1.0);
const DARK_GRAY_TEXT_COLOR   : color::Rgba = color::Rgba::new(1.0,1.0,1.0,0.6);

/// Default project name used by IDE on startup.
pub const DEFAULT_PROJECT_NAME:&str = "Unnamed";



// =============
// === Shape ===
// =============

mod shape {
    use super::*;

    ensogl::define_shape_system! {
            (style:Style, selection:f32) {
                let bg_color = color::Rgba::new(0.0,0.0,0.0,0.000_001);
                Plane().fill(bg_color).into()
            }
        }
}



// =================
// === FrpInputs ===
// =================

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct FrpInputs {
    /// Rename the project.
    pub rename     : frp::Source<String>,
    /// Reset the project name to the one before editing.
    pub reset_name : frp::Source<()>
}

impl FrpInputs {
    /// Create new FrpInputs.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! {network
            def reset_name = source();
            def rename     = source();
        }
        Self{reset_name,rename}
    }
}



// ==================
// === FrpOutputs ===
// ==================

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct FrpOutputs {
    /// Emits the new project name when it's renamed.
    pub renamed : frp::Source<String>
}

impl FrpOutputs {
    /// Create new FrpOutputs.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! {network
            def renamed = source();
        }
        Self{renamed}
    }
}

// ===========
// === Frp ===
// ===========

#[derive(Debug,Clone,CloneRef,Shrinkwrap)]
#[allow(missing_docs)]
pub struct Frp {
    #[shrinkwrap(main_field)]
    pub inputs  : FrpInputs,
    pub outputs : FrpOutputs,
    pub network : frp::Network,
}

impl Default for Frp {
    fn default() -> Self {
        let network = frp::Network::new();
        let inputs  = FrpInputs::new(&network);
        let outputs = FrpOutputs::new(&network);
        Self{network,inputs,outputs}
    }
}

impl Frp {
    /// Create new Frp.
    pub fn new() -> Self {
        default()
    }
}



// ==================
// === Animations ===
// ==================

/// ProjectName's animations handlers.
#[derive(Debug,Clone,CloneRef)]
pub struct Animations {
    highlight   : Animation<f32>,
    positioning : Animation<Vector3<f32>>
}

impl Animations {
    /// Create new animations handlers.
    pub fn new(network:&frp::Network) -> Self {
        let highlight   = Animation::<_>::new(&network);
        let positioning = Animation::<_>::new(&network);
        Self{highlight,positioning}
    }
}



// ========================
// === ProjectNameModel ===
// ========================

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct ProjectNameModel {
    logger         : Logger,
    animations     : Animations,
    view           : component::ShapeView<shape::Shape>,
    display_object : display::object::Instance,
    text_field     : TextField,
    project_name   : Rc<RefCell<String>>,
    renamed_output : frp::Source<String>,
}

impl ProjectNameModel {
    /// Create new ProjectNameModel.
    pub fn new(world:&World,frp:&Frp) -> Self {
        let scene                 = world.scene();
        let logger                = Logger::new("ProjectName");
        let display_object        = display::object::Instance::new(&logger);
        let font                  = scene.fonts.get_or_load_embedded_font("DejaVuSansMono").unwrap();
        let size                  = Vector2::new(600.0,100.0);
        let base_color            = DARK_GRAY_TEXT_COLOR;
        let text_size             = 16.0;
        let text_field_properties = TextFieldProperties{base_color,font,size,text_size};
        let text_field            = TextField::new(&world,text_field_properties);
        let view_logger           = Logger::sub(&logger,"view_logger");
        let view                  = component::ShapeView::<shape::Shape>::new(&view_logger,scene);
        let project_name          = Rc::new(RefCell::new(DEFAULT_PROJECT_NAME.to_string()));
        let renamed_output        = frp.outputs.renamed.clone();
        let animations            = Animations::new(&frp.network);
        Self{logger,view,display_object,text_field,project_name,renamed_output
            ,animations}.initialize()
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
        let mut height = default();
        self.text_field.with_content(|content| height = content.line_height);
        self.view.shape.sprite.size.set(Vector2::new(width,height));
        self.view.set_position(Vector3::new(0.0,-height/2.0,0.0));
        self.animations.positioning.set_target_value(offset);
    }

    fn initialize(self) -> Self {
        self.add_child(&self.text_field.display_object());
        self.add_child(&self.view);
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

        self.setup_text_field_content();
        self
    }

    fn reset_name(&self) {
        info!(self.logger, "Resetting project name.");
        self.setup_text_field_content();
    }

    fn setup_text_field_content(&self) {
        self.text_field.set_content(&self.project_name.borrow());
        self.setup_center_alignment();
    }

    fn highlight_text(&self) {
        self.animations.highlight.set_target_value(1.0);
    }

    fn darken_text(&self) {
        if !self.text_field.is_focused() {
            self.animations.highlight.set_target_value(0.0);
        }
    }

    fn set_highlight(&self, value:f32) {
        let base_color = linear_interpolation(DARK_GRAY_TEXT_COLOR,HIGHLIGHTED_TEXT_COLOR,value);
        self.text_field.set_base_color(base_color);
    }

    fn set_position(&self, value:Vector3<f32>) {
        self.text_field.set_position(value);
    }

    fn rename(&self, name:impl Str) {
        let name = name.into();
        *self.project_name.borrow_mut() = name.clone();
        self.setup_text_field_content();
        self.renamed_output.emit(name);
    }
}



// ===================
// === ProjectName ===
// ===================

/// The project name's view used for visualizing the project name and renaming it.
#[derive(Debug,Clone,CloneRef,Shrinkwrap)]
#[allow(missing_docs)]
pub struct ProjectName {
    #[shrinkwrap(main_field)]
    model   : Rc<ProjectNameModel>,
    pub frp : Frp
}

impl ProjectName {
    /// Create a new ProjectName view.
    pub fn new(world:&World) -> Self {
        let frp     = Frp::new();
        let model   = Rc::new(ProjectNameModel::new(world,&frp));
        let network = &frp.network;
        frp::extend! { network
            eval model.view.events.mouse_over((_) {model.highlight_text()});
            eval model.view.events.mouse_out ((_) {model.darken_text()});
            eval frp.inputs.reset_name((_) {model.reset_name()});
            eval frp.inputs.rename((name) {model.rename(name)});
        }

        // Animations

        frp::extend! {network
            eval model.animations.highlight.value((value) model.set_highlight(*value));
            eval model.animations.positioning.value((value) model.set_position(*value));
        }

        Self{frp,model}
    }
}

impl display::Object for ProjectNameModel {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}

impl display::Object for ProjectName {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
