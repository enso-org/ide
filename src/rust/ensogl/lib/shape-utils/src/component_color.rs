//! `ComponentColor` is an FRP based wrapper around colors that can change their state. Updates and
//! transitions of the color are emitted as FRP events nd make it easy to show animations of color
//! transitions. `ComponentColor` works both for static colors, as well as colors derived from a
//! theme.
//!
//! The `ComponentColor` keeps track of the original color and manages the querying of the style
//! and thus avoid boiler plate wherever we need to have colors that change between states.

use crate::prelude::*;

use enso_frp as frp;

use ensogl_core::application::Application;
use ensogl_core::data::color;
use ensogl_core::display::shape::*;
use ensogl_core::display::style::path::Path;
use ensogl_core::gui::component::Animation;
use ensogl_text;



// =================
// === Constants ===
// =================

const DEFAULT_COLOR    : color::Rgba = color::Rgba::new(1.0, 0.0, 0.0, 0.5);



// ====================
// === Color Source ===
// ====================

/// A `Source` contains the information required to get a color either from a theme, or statically
/// through a constant.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub enum Source {
    /// A constant color value.
    Static { color : color::Rgba },
    /// A color derived from the current theme.
    /// Should update automatically once #795 is resolved.
    Theme  { path  : Path }
}

impl Default for Source {
    fn default() -> Self {
        Source::Static{color:DEFAULT_COLOR}
    }
}

impl<C> From<color::Color<C>> for Source
    where color::Color<C> : Into<color::Rgba> {
    fn from(color:color::Color<C>) -> Source {
        let color = color.into();
        Source::Static {color}
    }
}

impl From<Path> for Source {
    fn from(path:Path) -> Self {
        Source::Theme {path}
    }
}



// ===================
// === Color State ===
// ===================

/// Indicates which state of the color should be displayed.
///
/// Note: Can be extended with new states in the future, for example, "Highlight" or "Achromatic".
#[derive(Clone,Copy,Debug)]
pub enum State {
    /// The default color state.
    Base,
    /// A dimmed, muted version of the color. Used to indicate inactivity.
    Dim,
    /// Invisible color with alpha set to 0.
    Transparent,
}

impl Default for State {
    fn default() -> Self {
        State::Base
    }
}



// ===========
// === Frp ===
// ===========

ensogl_text::define_endpoints! {
    Input {
        source (Source),
        state  (State),
    }
    Output {
        color  (color::Rgba),
    }
}



// =============
// === Model ===
// =============

#[derive(Clone,Debug)]
struct Model {
    color_source : RefCell<Source>,
    // FIXME[MM] : Replace style watch when #795 is resolved with whatever replaces it.
    styles       : StyleWatch
}

impl Model {
    fn new(app:&Application) -> Self {
        let color_source = default();
        let styles       = StyleWatch::new(&app.display.scene().style_sheet);
        Self{color_source,styles}
    }

    fn set_source(&self, source:Source) {
        self.color_source.replace(source);
    }

    fn get_base_color(&self) -> color::Lcha {
        match self.color_source.borrow().clone() {
            Source::Static{color} => color.into(),
            Source::Theme{path}   => self.styles.get_color(path),
        }
    }

    /// Return the modified version of the base color.
    fn get_color_dim(&self) -> color::Lcha {
        match self.color_source.borrow().clone() {
            Source::Static{color} => {
                self.styles.make_color_dim(color).into()
            },
            Source::Theme{path}   => {
                self.styles.get_color_dim(path)
            },
        }
    }

    fn get_color_transparent(&self) -> color::Lcha {
        let mut base    = self.get_base_color();
        base.data.alpha = 0.0;
        base
    }
}



// =====================
// === Dynamic Color ===
// =====================

/// The `DynamicColor` provides color information through an FRP api. It allows for dynamic color
/// transitions between different states (e.g, dim or not dim) that are emitted like an animation.
#[derive(Clone,CloneRef,Debug)]
pub struct ComponentColor {
    /// Public FRP api.
    pub frp   : Frp,
        model : Rc<Model>,
}


impl ComponentColor {

    /// Constructor.
    pub fn new(app:&Application) -> Self {
        let frp   = Frp::new_network();
        let model = Rc::new(Model::new(app));
        Self{frp,model}.init()
    }

    fn init(self) -> Self {
        let network = &self.frp.network;
        let frp     = &self.frp;
        let model   = &self.model;

        let color = Animation::<color::Lcha>::new(&network);

        frp::extend! { network
            color_parameters <- all(frp.input.source,frp.input.state);
            eval color_parameters ([model,color]((source,state)){
                model.set_source(source.clone());
                let target_color = match *state {
                    State::Base => {
                        model.get_base_color()
                    },
                    State::Dim => {
                        model.get_color_dim()
                    }
                    State::Transparent => {
                        model.get_color_transparent()
                    }
                };
                color.set_target_value(target_color);
            });

            frp.source.color <+ color.value.map(|v| {
                v.into()
            });
        }

        frp.state.emit(State::Base);

        self
    }
}
