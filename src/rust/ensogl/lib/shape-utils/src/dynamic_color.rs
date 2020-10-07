//! `DynamicColor` is a FRP based wrapper around colors that can change their state. Updates and
//! transitions of the color are emitted as FRP events nd make it easy to show animations of color
//! transitions. `DynamicColor` works both for static colors, as well as colors derived from a
//! theme.

use crate::prelude::*;

use enso_frp as frp;
use enso_frp;


use ensogl_core::application::Application;
use ensogl_core::data::color;
use ensogl_core::display::shape::*;
use ensogl_core::display::style::data::DataMatch;
use ensogl_core::display::style::path::Path;
use ensogl_core::gui::component::Animation;
use ensogl_text;
use ensogl_theme as theme;



// =================
// === Constants ===
// =================

const DEFAULT_COLOR    : color::Rgba = color::Rgba::new(1.0, 0.0, 0.0, 0.5);
/// Key that is used to look for a dim variant of a color in the theme.
const THEME_KEY_DIMMED : &str = " dimmed";



// ====================
// === Color Mixing ===
// ====================

/// Linear interpolation between two numbers.
fn lerp(a:f32, b:f32, value:f32) -> f32 {
    b * value + a * (1.0-value )
}

/// Linear mixing of two colors in Lcha color space.
/// TODO consider refining and moving to the main color module.
fn mix<T1:Into<color::Lcha>,T2:Into<color::Lcha>>(color_a:T1,color_b:T2,mix_value:f32) -> color::Lcha {
    let color_a   = color_a.into();
    let color_b   = color_b.into();
    let lightness = lerp(color_a.lightness,color_b.lightness,mix_value);
    let chroma    = lerp(color_a.chroma,color_b.chroma,mix_value);
    // TODO check whether hue needs to be done differently for shortest path
    let hue       = lerp(color_a.hue,color_b.hue,mix_value);
    let alpha     = lerp(color_a.alpha,color_b.alpha,mix_value);
    color::Lcha::new(lightness,chroma,hue,alpha)
}



// ====================
// === Color Source ===
// ====================

/// A `Source` contains the information required to get a color either from a theme, or statically
/// through a constant.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub enum Source {
    /// A constant color value.
    Static { color : color::Rgba},
    /// A color derived from the current theme.
    /// Should update automatically once #795 is resolved.
    Theme  { path  : Path}
}

impl Default for Source {
    fn default() -> Self {
        Source::Static{color:DEFAULT_COLOR}
    }
}

impl From<color::Rgba> for Source {
    fn from(color:color::Rgba) -> Self {
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
/// Note: ca be extended with new states in the future, for example, "Highlight" or "Achromatic"
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
        set_source (Source),
        set_state  (State),
    }
    Output {
        color                (color::Rgba),
    }
}



// =============
// === Model ===
// =============

#[derive(Clone,Debug)]
struct Model {
    color_source:  RefCell<Source>,
    // FIXME : Replace style watch when #795 is resolved with whatever replaces it.
    styles       : StyleWatch
}

impl Model {
    fn new(app:&Application) -> Self {
        let color_path  = default();
        let styles      = StyleWatch::new(&app.display.scene().style_sheet);
        Self{ color_source: color_path,styles}
    }

    fn set_source(&self, source:Source) {
        self.color_source.replace(source);
    }

    /// Return the path where we look for variant colors in the theme.
    fn variant_path(path:Path, extension:String) -> Path {
        let segments_rev = path.rev_segments;
        let mut segments = segments_rev.into_iter().rev().collect_vec();
        segments.pop();
        segments.push(" variant ".to_string());
        segments.push(extension);
        Path::from_segments(segments)
    }

    fn try_get_color_variant_from_theme(&self, id:&str) -> Option<color::Rgba> {
        if let Source::Theme{ path } = self.color_source.borrow().clone() {
            let path  = Self::variant_path(path,id.to_string());
            let color = self.styles.get(path).color()?;
            Some(color::Rgba::from(color))
        } else {
            None
        }
    }

    fn get_base_color(&self) -> color::Rgba {
        match self.color_source.borrow().clone() {
            Source::Static{color} => color,
            Source::Theme{path}   => self.styles.get_color(path).into(),
        }
    }

    /// Create a dimmed version of the given color value. The exact values to be used for dimming
    /// are derived from the theme.
    fn make_dimmed_color(&self, color:color::Rgba) -> color::Rgba {
        let color : color::Lcha    = color.into();
        let color_lightness_factor = theme::vars::graph_editor::colors::default::dimming::lightness_factor;
        let color_chroma_factor    = theme::vars::graph_editor::colors::default::dimming::chroma_factor;
        let color_lightness_factor = self.styles.get_number_or(color_lightness_factor,0.0);
        let color_chroma_factor    = self.styles.get_number_or(color_chroma_factor,0.0);
        let lightness              = color.lightness * color_lightness_factor;
        let chroma                 = color.chroma * color_chroma_factor;
        let color                  = color::Lcha::new(lightness,chroma,color.hue,color.alpha);
        color.into()
    }

    /// Return the modified version of the base color.
    fn get_parametrized_color(&self, dimmnes:f32, alpha:f32) -> color::Rgba {
        let base_color = self.get_base_color();
        // Check whether there is a version defined in the theme, otherwise we create our own.
        let dimmed_color = match self.try_get_color_variant_from_theme(THEME_KEY_DIMMED) {
            None        => self.make_dimmed_color(base_color),
            Some(color) => color,
        };
        let mut color = mix(base_color,dimmed_color,dimmnes);
        color.data.alpha *= alpha;
        color.into()
    }
}



// =====================
// === Dynamic Color ===
// =====================

/// The `DynamicColor` provides color information through an FRP api. It allows for dynamic color
/// transitions between different states (e.g, dim or not dim) that are emitted like an animation.
#[derive(Clone,CloneRef,Debug)]
pub struct DynamicColor {
    /// Public FRP api.
    pub frp   : Frp,
        model : Rc<Model>,
}


impl DynamicColor {

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

        let dimmnes          = Animation::<f32>::new(&network);
        let alpha            = Animation::<f32>::new(&network);

        alpha.simulator.set_value(1.0);
        dimmnes.simulator.set_value(0.0);

        frp::extend! { network
            eval frp.set_state([dimmnes,alpha] (state) {
                match *state {
                    State::Base => {
                        dimmnes.set_target_value(0.0);
                        alpha.set_target_value(1.0);
                    },
                    State::Dim => {
                        dimmnes.set_target_value(1.0);
                        alpha.set_target_value(1.0);
                    }
                    State::Transparent => {
                        alpha.set_target_value(0.0);
                    }
                }
            });

            color_parameters <- all(frp.set_source,dimmnes.value,alpha.value);
            color <- color_parameters.map(f!([model]((source,value,alpha)){
                  model.set_source(source.clone());
                  model.get_parametrized_color(*value,*alpha)
            }));
            frp.source.color <+ color;
        }

        frp.set_state(State::Base);

        self
    }
}
