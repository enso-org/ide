

// =================
// === Constants ===
// =================

#[macro_export]
macro_rules! hover_rect {
    () => {
        use ensogl::data::color;
        use ensogl::display::shape::*;

        /// Invisible rectangular area that can be hovered.
        mod hover_rect {

            ensogl::define_shape_system! {
                (corner_radius:f32) {
                    let width  : Var<Pixels> = "input_size.x".into();
                    let height : Var<Pixels> = "input_size.y".into();
                    let rect           = Rect((&width,&height));
                    let rect_rounded   = rect.corner_radius(corner_radius);
                    let rect_filled    = rect_rounded.fill(HOVER_COLOR);
                    rect_filled.into()
                }
            }
        }
    }

}
