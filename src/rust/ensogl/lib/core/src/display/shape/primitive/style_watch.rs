//! Style watch utility. Allows querying of the current style for information.

use crate::prelude::*;

use crate::control::callback;
use crate::data::color;
use crate::display::style::Path;
use crate::display::style::data::DataMatch;
use crate::display::style;



// =================
// === Constants ===
// =================

/// Key that is used to look for a dim variant of a color in the theme.
const THEME_KEY_DIMMED            : &str = " dimmed";
const VARIANT_PATH_PREFIX         : &str = " variant ";
const COLOR_LIGHTNESS_FACTOR_PATH : &str = "theme.vars.colors.dimming.lightness_factor";
const COLOR_CHROMA_FACTOR_PATH    : &str = "theme.vars.colors.dimming.chroma_factor";



// ==================
// === StyleWatch ===
// ==================


/// Style watch utility. It's reference is passed to shapes defined with the `define_shape_system`
/// macro. Whenever a style sheet value is accessed, the value reference is being remembered and
/// tracked. Whenever it changes, the `callback` runs. The callback should trigger shape redraw.
#[derive(Clone,CloneRef,Derivative)]
#[derivative(Debug)]
pub struct StyleWatch {
    sheet    : style::Sheet,
    vars     : Rc<RefCell<Vec<style::Var>>>,
    handles  : Rc<RefCell<Vec<callback::Handle>>>,
    #[derivative(Debug="ignore")]
    callback : Rc<RefCell<Box<dyn Fn()>>>,
}

impl StyleWatch {
    /// Constructor.
    #[allow(trivial_casts)]
    pub fn new(sheet:&style::Sheet) -> Self {
        let sheet    = sheet.clone_ref();
        let vars     = default();
        let handles  = default();
        let callback = Rc::new(RefCell::new(Box::new(||{}) as Box<dyn Fn()>));
        Self {sheet,vars,handles,callback}
    }

    /// Resets the state of style manager. Should be used on each new shape definition. It is
    /// called automatically when used by `define_shape_system`.
    pub fn reset(&self) {
        *self.vars.borrow_mut()    = default();
        *self.handles.borrow_mut() = default();
    }

    /// Queries style sheet value for a value.
    pub fn get<T:Into<Path>>(&self, path:T) -> Option<style::Data> {
        let var      = self.sheet.var(path);
        let value    = var.value();
        let callback = self.callback.clone_ref();
        var.on_change(move |_:&Option<style::Data>| (callback.borrow())());
        self.vars.borrow_mut().push(var);
        value
    }

    /// Sets the callback which will be used when dependent styles change.
    pub fn set_on_style_change<F:'static+Fn()>(&self, callback:F) {
        *self.callback.borrow_mut() = Box::new(callback);
    }

    /// Queries style sheet number value, if not found gets fallback.
    pub fn get_number_or(&self, path:&str, fallback:f32) -> f32 {
        self.get(path).number().unwrap_or(fallback)
    }
}



// ====================
// === Color Styles ===
// ====================


impl StyleWatch {
    /// Queries style sheet color, if not found fallbacks to red.
    pub fn get_color<T:Into<Path>>(&self, path:T) -> color::Lcha {
        let fallback = color::Rgba::new(1.0,0.0,0.0,1.0).into();
        self.get(path).color().unwrap_or_else(|| fallback)
    }

    /// Queries style sheet color, if not found fallbacks to red.
    pub fn get_color_dim<T:Into<Path>>(&self, path:T) -> color::Lcha {
        let path = path.into();
        match self.try_get_color_variant(path.clone(),THEME_KEY_DIMMED) {
            None        => {
                let base_color = self.get_color(path);
                self.make_color_dim(base_color)
            },
            Some(color) => color,
        }
    }

    /// Create a dimmed version of the given color value. The exact values to be used for dimming
    /// are derived from the theme.
    pub fn make_color_dim<T:Into<color::Lcha>+From<color::Lcha>>(&self, color:T) -> T {
        let color : color::Lcha    = color.into();
        let color_lightness_factor = self.get_number_or(COLOR_LIGHTNESS_FACTOR_PATH, 0.0);
        let color_chroma_factor    = self.get_number_or(COLOR_CHROMA_FACTOR_PATH, 0.0);
        let lightness              = color.lightness * color_lightness_factor;
        let chroma                 = color.chroma * color_chroma_factor;
        let color                  = color::Lcha::new(lightness,chroma,color.hue,color.alpha);
        color.into()
    }

    /// Return the path where we look for alternative shades or scheme variants of a color in the
    /// theme (for example, "dimmed").
    fn color_variant_path(path:Path, extension:String) -> Path {
        let segments_rev = path.rev_segments;
        let mut segments = segments_rev.into_iter().rev().collect_vec();
        segments.pop();
        segments.push(VARIANT_PATH_PREFIX.to_string());
        segments.push(extension);
        Path::from_segments(segments)
    }

    fn try_get_color_variant<T:Into<Path>>(&self, path:T, id:&str) -> Option<color::Lcha> {
        let path  = Self::color_variant_path(path.into(), id.to_string());
        self.get(path).color()
    }

}
