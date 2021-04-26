use crate::prelude::*;

use ensogl_core::display::shape::*;
use ensogl_theme as theme;

use crate::shadow;



// ==================
// === Background ===
// ==================

/// Utility struct that contains the background shape for the selector components, as well as some
/// meta information about it. This information can be used to align other shapes with the
/// background.
#[allow(dead_code)]
struct Background {
    width         : Var<Pixels>,
    height        : Var<Pixels>,
    corner_radius : Var<Pixels>,
    shape         : AnyShape,
}

impl Background {
    fn new(corner_left:&Var<f32>,corner_right:&Var<f32>,style:&StyleWatch) -> Background {
        let sprite_width  : Var<Pixels> = "input_size.x".into();
        let sprite_height : Var<Pixels> = "input_size.y".into();
        let width         = &sprite_width - shadow::size(style).px();
        let height        = &sprite_height - shadow::size(style).px();
        let corner_radius = &height/2.0;
        let rect_left     = Rect((&width/2.0,&height)).corners_radius(&corner_radius*corner_left);
        let rect_left     = rect_left.translate_x(-&width/4.0);
        let rect_right    = Rect((&width/2.0,&height)).corners_radius(&corner_radius*corner_right);
        let rect_right    = rect_right.translate_x(&width/4.0);
        let rect_center   = Rect((&corner_radius,&height));

        let shape = (rect_left+rect_right+rect_center).into();

        Background{width,height,corner_radius,shape}
    }
}

/// Background shape. Appears as a rect with rounded corners. The roundness of each corner can be
/// toggled by passing `0.0` or `1.0` to either `corner_left` and `corner_right` where `0.0` means
/// "not rounded" and `1.0` means "rounded".
pub mod background {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style,corner_left:f32,corner_right:f32) {
            let background = Background::new(&corner_left,&corner_right,style);
            let color      = style.get_color(theme::component::slider::background);
            let shadow     = shadow::from_shape(background.shape.clone(),style);
            let background = background.shape.fill(color);
            (shadow + background).into()
        }
    }
}



// ===============
// === IO Rect ===
// ===============

/// Utility shape that is invisible but provides mouse input. Fills the whole sprite.
pub mod io_rect {
    use super::*;

    ensogl_core::define_shape_system! {
        () {
            let sprite_width  : Var<Pixels> = "input_size.x".into();
            let sprite_height : Var<Pixels> = "input_size.y".into();
            let rect          = Rect((&sprite_width,&sprite_height));
            let shape         = rect.fill(HOVER_COLOR);
            shape.into()
        }
    }
}



// =============
// === Track ===
// =============

/// Track of the selector. Appears as filled area of the background. Has a definable start and
/// end-point (`left`, `right`) which are passed as normalised values relative to the maximum
/// width.  For consistency with the background shape, also has the property to round either side
/// of the track, when required to fit the background shape. (See `Background` above.).
pub mod track {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style,left:f32,right:f32,corner_left:f32,corner_right:f32) {
            let background    = Background::new(&corner_left,&corner_right,style);
            let width         = background.width;
            let height        = background.height;

            let track_width  = (&right - &left) * &width;
            let track_start  = left * &width;
            let track        = Rect((&track_width,&height));
            let track        = track.translate_x(&track_start + (track_width / 2.0) );
            let track        = track.translate_x(-width/2.0);
            let track        = track.intersection(&background.shape);

            let track_color  = style.get_color(theme::component::slider::track::color);
            let track        = track.fill(track_color);
            track.into()
          }
    }
}



// ================
// === OverFlow ===
// ================

/// Utility struct that contains the overflow shape, and some metadata that can be used to place and
/// align it.
#[allow(dead_code)]
struct OverFlowShape {
    width  : Var<Pixels>,
    height : Var<Pixels>,
    shape  : AnyShape
}

impl OverFlowShape {
    fn new(style:&StyleWatch) -> Self {
        let sprite_width  : Var<Pixels> = "input_size.x".into();
        let sprite_height : Var<Pixels> = "input_size.y".into();
        let width         = &sprite_width - shadow::size(style).px();
        let height        = &sprite_height - shadow::size(style).px();
        let overflow_color  = style.get_color(theme::component::slider::overflow::color);
        let shape  = Triangle(&sprite_height/6.0,&sprite_height/6.0);
        let shape  = shape.fill(&overflow_color);

        let hover_area = Circle(&height);
        let hover_area = hover_area.fill(HOVER_COLOR);

        let shape = (shape + hover_area).into();
        OverFlowShape{shape,width,height}
    }
}

/// Overflow shape that indicates a value can not be shown. Appears as a triangle/arrow pointing
/// left.
pub mod left_overflow {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style) {
            let overflow_shape = OverFlowShape::new(style);
            let shape = overflow_shape.shape.rotate(-90.0_f32.to_radians().radians());
            shape.into()
          }
    }
}

/// Overflow shape that indicates a value can not be shown. Appears as a triangle/arrow pointing
/// right.
pub mod right_overflow {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style) {
            let overflow_shape = OverFlowShape::new(style);
            let shape = overflow_shape.shape.rotate(90.0_f32.to_radians().radians());
            shape.into()
          }
    }
}
