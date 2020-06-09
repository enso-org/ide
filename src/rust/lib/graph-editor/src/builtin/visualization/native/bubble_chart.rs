//! Bubble Chart visualisation implemented using the native shape system.

use crate::prelude::*;

use crate::component::visualization::*;
use crate::component::visualization;
use crate::frp;

use ensogl::data::color::Rgba;
use ensogl::display::scene::Scene;
use ensogl::display;
use ensogl::gui::component;



// ===================
// === BubbleChart ===
// ===================

/// Bubble shape definition.
pub mod shape {
    use super::*;
    use ensogl::display::shape::*;
    use ensogl::display::scene::Scene;
    use ensogl::display::Sprite;
    use ensogl::display::Buffer;
    use ensogl::display::Attribute;

    ensogl::define_shape_system! {
        (position:Vector2<f32>,radius:f32) {
            let node = Circle(radius);
            let node = node.fill(Rgba::new(0.17,0.46,0.15,1.0));
            let node = node.translate(("input_position.x","input_position.y"));
            node.into()
        }
    }
}

/// Sample implementation of a Bubble Chart.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct BubbleChart {
    pub display_object : display::object::Instance,
    pub scene          : Scene,
    signature      : Signature,
    frp            : visualization::instance::Frp,
    views          : RefCell<Vec<component::ShapeView<shape::Shape>>>,
    logger         : Logger,
    size           : Rc<Cell<V2>>,
}

#[allow(missing_docs)]
impl BubbleChart {
    pub fn definition() -> Definition {
        Definition::new(
            Signature::new_for_any_type(Path::builtin("[Demo] Bubble Visualization")),
            |scene| { Ok(Self::new(scene).into()) }
        )
    }

    pub fn new(scene:&Scene) -> Self {
        let logger         = Logger::new("bubble");
        let display_object = display::object::Instance::new(&logger);
        let views          = RefCell::new(vec![]);
        let frp            = visualization::instance::Frp::new();
        let size           = default();
        let scene          = scene.clone_ref();
        let signature      = Signature::new_for_any_type(Path::builtin("[Demo] Bubble Chart"));
        BubbleChart {display_object,views,logger,frp,size,scene,signature} . init()
    }

    fn init(self) -> Self {
        let network = &self.frp.network;
        let size    = &self.size;
        frp::extend! { network
            eval self.frp.set_size ((s) size.set(*s));
            eval self.frp.send_data ([](_data) {
            // FIXME: uncomment and update.
//                let data_inner: Rc<Vec<Vector3<f32>>> = data.as_binary()?;
//                // Avoid re-creating views, if we have already created some before.
//                let mut views = self.views.borrow_mut();
//                views.resize_with(data_inner.len(),|| component::ShapeView::new(&self.logger,&self.scene));
//
//                // TODO[mm] this is somewhat inefficient, as the canvas for each bubble is too large.
//                // But this ensures that we can get a cropped view area and avoids an issue with the data
//                // and position not matching up.
//                views.iter().zip(data_inner.iter()).for_each(|(view,item)| {
//                    let size : Vector2<f32> = self.size.get().into();
//                    view.display_object.set_parent(&self.display_object);
//                    view.shape.sprite.size().set(size);
//                    view.shape.radius.set(item.z);
//                    view.shape.position.set(Vector2::new(item.x,item.y) - size / 2.0);
//                });
//                Ok(())
            });
        }
        self
    }
}

impl From<BubbleChart> for Instance {
    fn from(t:BubbleChart) -> Self {
        Self::new(&t,&t.frp)
    }
}

impl display::Object for BubbleChart {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object.display_object()
    }
}

