use crate::prelude::*;

use ensogl::data::color;
use ensogl::display::scene::Scene;
use ensogl::display::shape::*;
use ensogl::gui::component;
use ensogl::display;

use crate::node::input::area;



// ===================
// === Hover Shape ===
// ===================

/// Port shape definition.
pub mod hover {
    use super::*;
    ensogl::define_shape_system! {
        (style:Style) {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let shape  = Rect((&width,&height));
            if !area::DEBUG {
                let color = Var::<color::Rgba>::from("srgba(1.0,1.0,1.0,0.00001)");
                shape.fill(color).into()
            } else {
                let shape = shape.corners_radius(6.px());
                let color = Var::<color::Rgba>::from("srgba(1.0,0.0,0.0,0.1)");
                shape.fill(color).into()
            }
        }
    }
}

/// Function used to hack depth sorting. To be removed when it will be implemented in core engine.
pub fn depth_sort_hack(scene:&Scene) {
    let logger = Logger::new("hack");
    component::ShapeView::<hover::Shape>::new(&logger,scene);
}



// =============
// === Shape ===
// =============

/// Port shape definition.
pub mod viz {
    use super::*;
    ensogl::define_shape_system! {
        (style:Style) {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let shape  = Rect((&width,&height)).corners_radius(&height / 2.0);
            let color  = Var::<color::Rgba>::from("srgba(1.0,0.0,0.0,1.0)");
            shape.fill(color).into()
        }
    }
}



// ==============
// === Shapes ===
// ==============

/// Shapes the port is build from. It consist of the `hover_shape`, which represents a hover area of
/// a full node height, and the `viz_shape`, which is a nice, visual highlight representation.
/// Both shapes are children of the `root` display object:
///
///     hover_shape
///      ◄──────►
/// ╭───┬────────┬──┄
/// │   │╭──────╮│▼ viz_shape
/// │   │╰──────╯│▲ (appears after mouse_hover)
/// ╰───┴────────┴──┄
///
#[derive(Clone,CloneRef,Debug)]
pub struct Shapes {
    pub root  : display::object::Instance,
    pub hover : component::ShapeView<hover::Shape>,
    pub viz   : component::ShapeView<viz::Shape>,
}

impl Shapes {
    pub fn new(logger:&Logger, scene:&Scene) -> Self {
        let root  = display::object::Instance::new(logger);
        let hover = component::ShapeView::<hover::Shape>::new(logger,scene);
        let viz   = component::ShapeView::<viz::Shape>::new(logger,scene);
        root.add_child(&hover);
        root.add_child(&viz);
        let viz_shape_system = scene.shapes.shape_system(PhantomData::<viz::Shape>);
        viz_shape_system.shape_system.set_pointer_events(false);
        Self {root,hover,viz}
    }
}



// =============
// === Model ===
// =============

ensogl::define_endpoints! {
    Input {
        set_optional         (bool),
        set_disabled         (bool),
        set_hover            (bool),
        set_connected        (bool),
        set_parent_connected (bool),
    }

    Output {
        color (color::Lcha)
    }
}

/// Input port model.
#[derive(Clone,Debug,Default)]
pub struct Model {
    pub frp         : Frp,
    pub shapes      : Option<Shapes>,
    pub name        : Option<String>,
    pub index       : usize,
    pub local_index : usize,
    pub length      : usize,
    pub color       : color::Animation,
}

impl Deref for Model {
    type Target = Frp;
    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}

impl Model {
    pub fn new() -> Self {
        default()
    }

    /// Shapes initialization. Please note that not all port models get their shapes initialized,
    /// as some are skipped. For example, given the expression `(((foo)))`, the inner parentheses
    /// will be skipped, as there is no point in making them ports. The skip algorithm is
    /// implemented as part of the port are initialization.
    pub fn init_shapes(&mut self, logger:impl AnyLogger, scene:&Scene) -> &Shapes {
        let logger_name  = format!("port({},{})",self.index,self.length);
        let logger       = Logger::sub(logger,logger_name);
        let shapes       = Shapes::new(&logger,scene);
        self.shapes      = Some(shapes);
        &self.shapes.as_ref().unwrap()
    }
}
