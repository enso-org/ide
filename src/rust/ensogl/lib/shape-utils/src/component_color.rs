//! `ComponentColor` is a thin FRP based wrapper around animated colors that can change their state.
//! Updates and transitions of the color are emitted as FRP events.
//!
//! Using the `ComponentColor` instead of the animation directly reduces boilerplate
//! (for example to keep track of the original color) and make it easy to show pleasant animations
//! of color transitions. `ComponentColor` works both for static colors, as well as colors derived
//! from a theme.

use crate::prelude::*;

use enso_frp as frp;

use ensogl_core::application::Application;
use ensogl_core::data::color;
use ensogl_core::display::shape::*;
use ensogl_core::display::style::path::Path;
use ensogl_core::gui::component::Animation;



// =================
// === Constants ===
// =================

const DEFAULT_COLOR : color::Rgba = color::Rgba::new(1.0, 0.0, 0.0, 0.5);



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

ensogl_core::define_endpoints! {
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
    color_source : RefCell<Option<Source>>,
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
        self.color_source.replace(Some(source));
    }

    fn get_base_color(&self) -> color::Lcha {
        match self.color_source.borrow().clone() {
            Some(Source::Static{color}) => color.into(),
            Some(Source::Theme{path})   => self.styles.get_color(path),
            None                        => DEFAULT_COLOR.into()
        }
    }

    /// Return the modified version of the base color.
    fn get_color_dim(&self) -> color::Lcha {
        match self.color_source.borrow().clone() {
            Some(Source::Static{color}) => {
                self.styles.make_color_dim(color).into()
            },
            Some(Source::Theme{path})   => {
                self.styles.get_color_dim(path)
            },
            None => DEFAULT_COLOR.into(),
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

            source_update <- frp.input.source.map(f!([model,color](source) {
                // Init color right away to avoid a "from black" animation on startup.
                if model.color_source.borrow().is_none() {
                    model.set_source(source.clone());
                    color.set_value(model.get_base_color());
                } else {
                    // Update source
                    model.set_source(source.clone());
                }
            }));

            color_parameters <- all(source_update,frp.input.state);
            eval color_parameters ([model,color]((_,state)){
                // Set up animation
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

        // frp.state.emit(State::Base);

        self
    }
}
