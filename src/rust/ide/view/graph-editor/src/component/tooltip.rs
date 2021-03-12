//! The `Tooltip` shows extra information for UI components. It is pegged to the cursor location
//! and appears when it receives information to show.

use enso_frp as frp;
use ensogl::application::Application;
use ensogl::display;
use ensogl::prelude::*;
use ensogl::gui::style::*;
use ensogl::define_style;

use ensogl_gui_components::label::Label;
use ensogl::animation::hysteretic::HystereticAnimation;



// =================
// === Constants ===
// =================

const HIDE_DELAY_DURATION_MS : f32 = 150.0;
const SHOW_DELAY_DURATION_MS : f32 = 150.0;



// =============
// === Style ===
// =============

define_style! {
    /// Host defines an object which the cursor position is bound to. It is used to implement
    /// label selection. After setting the host to the label, cursor will not follow mouse anymore,
    /// it will inherit its position from the label instead.
    text : Option<String>,
}

impl Style {
    /// Create a `TooltipUpdate` that sets the label of the tooltip.
    pub fn set_label(text:String) -> Self {
        let text = Some(StyleValue::new(Some(text)));
        Self{text}
    }
    /// Create a `TooltipUpdate` that unsets the label of the tooltip.
    pub fn unset_label() -> Self {
        let text = Some(StyleValue::new(None));
        Self{text}
    }

    fn has_text(&self) -> bool {
        if let Some(style_value) = self.text.as_ref() {
            if let Some(inner) = style_value.value.as_ref() {
                return inner.is_some()
            }
        }
        false
    }
}



// ==============
// === Offset ===
// ==============

#[derive(Clone,Copy,Debug)]
#[allow(missing_docs)]
/// Indicates the placement of the tooltip relative to the base position location.
pub enum Placement {
    Top, Bottom, Left, Right
}

impl Default for Placement {
    fn default() -> Self {
        Placement::Top
    }
}

ensogl::define_endpoints! {
    Input {
        set_style    (Style),
        set_location (Vector2),
        set_offset   (Placement)
    }
}



// =============
// === Model ===
// =============

#[derive(Clone,Debug)]
struct Model {
    tooltip : Label,
    root    : display::object::Instance,
}

impl Model {
    fn new(app:&Application) -> Self {
        let logger  = Logger::new("TooltipModel");
        let tooltip = Label::new(app.clone_ref());
        let root    = display::object::Instance::new(&logger);
        root.add_child(&tooltip);

        Self{tooltip,root}
    }

    fn set_location(&self, position:Vector2, size:Vector2, layout: Placement) {
        let layout_offset = match layout {
            Placement::Top    => Vector2::new(0.0, size.y * 0.5),
            Placement::Bottom => Vector2::new(0.0, -size.y * 0.5),
            Placement::Left   => Vector2::new(-size.x / 2.0, 0.0),
            Placement::Right  => Vector2::new(size.x / 2.0, 0.0),
        };

        let base_positions = position.xy();
        self.tooltip.set_position_xy(base_positions + layout_offset)
    }

    fn set_style(&self, update:&Style) {
        if let Some(style) = update.text.as_ref() {
            let text = style.value.clone().flatten();
            if let Some(text) = text {
                self.tooltip.frp.set_content(text)
            }
        }
    }

    fn set_visibility(&self, visible:bool) {
        if visible {
            self.root.add_child(&self.tooltip)
        } else {
            self.tooltip.unset_parent()
        }
    }

    fn set_opacity(&self, value:f32) {
        self.tooltip.frp.set_opacity(value);
    }
}



// ===============
// === Tooltip ===
// ===============

/// Tooltip component that can show extra information about other UI components.
#[derive(Clone,CloneRef,Debug)]
pub struct Tooltip {
    model   : Rc<Model>,
    #[allow(missing_docs)]
    pub frp : Rc<Frp>
}

impl Tooltip {
    #[allow(missing_docs)]
    pub fn new(app: &Application) -> Self {
        let model = Rc::new(Model::new(app));
        let frp   = Rc::new(Frp::new());
        Self{model,frp}.init()
    }

    fn init(self,) -> Self {
        let frp     = &self.frp;
        let network = &frp.network;
        let model   = &self.model;

        let hysteretic_transition = HystereticAnimation::new(
            &network,HIDE_DELAY_DURATION_MS,SHOW_DELAY_DURATION_MS);

        frp::extend! { network


            // === Style ===

             eval frp.set_style ((t) model.set_style(t));
             show_text         <- frp.set_style.map(|c| c.has_text());
             on_has_content    <- show_text.on_true();
             on_has_no_content <- show_text.on_false();


            // === Location ===

             location_update <- all(frp.set_location,model.tooltip.frp.size,frp.set_offset);
             eval location_update (((pos,size,offset)) model.set_location(*pos,*size,*offset));


            // === Transition ===

             hysteretic_transition.to_start <+ on_has_content;
             hysteretic_transition.to_end   <+ on_has_no_content;

             eval hysteretic_transition.value ([model](value) {
                model.set_opacity(*value);
                if *value >= 0.0 {
                    model.set_visibility(true)
                } else if *value <= 0.0 {
                    model.set_visibility(false)
                }
             });
        }

        // Reset appearance to avoid artifacts when first shown.
        model.set_opacity(0.0);
        model.set_style(&Style::default());
        self
    }
}

impl display::Object for Tooltip {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.root.display_object()
    }
}
