//! Bubble Chart visualisation implemented using the native shape system.

use crate::prelude::*;

use crate::component::visualization::*;
use crate::component::visualization;
use crate::frp;

use ensogl::data::color::Rgba;
use ensogl::display::scene::Scene;
use ensogl::display;
use ensogl::gui::component;



// =============
// === Shape ===
// =============

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



// ========================
// === BubbleChartModel ===
// ========================

/// Sample implementation of a Bubble Chart.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct BubbleChartModel {
    pub display_object : display::object::Instance,
    pub scene          : Scene,
    signature          : Signature,
    views              : Rc<RefCell<Vec<component::ShapeView<shape::Shape>>>>,
    logger             : Logger,
    size               : Rc<Cell<Vector2>>,
}

impl BubbleChartModel {
    fn receive_data(&self, data:&Data) -> Result<(),DataError> {
        let data_inner = match data {
            Data::Json {content} => content,
            _ => todo!() // FIXME
        };
        let data_inner:&serde_json::Value = data_inner;
        let data_inner: Rc<Vec<Vector3<f32>>> = if let Ok(result) = serde_json::from_value(data_inner.clone()) {
            result
        } else {
            return Err(DataError::InvalidDataType)
        };

        // Avoid re-creating views, if we have already created some before.
        let mut views = self.views.borrow_mut();
        views.resize_with(data_inner.len(),|| component::ShapeView::new(&self.logger,&self.scene));

        // TODO[mm] this is somewhat inefficient, as the canvas for each bubble is too large.
        // But this ensures that we can get a cropped view area and avoids an issue with the data
        // and position not matching up.
        views.iter().zip(data_inner.iter()).for_each(|(view,item)| {
            let size = self.size.get();
            view.display_object.set_parent(&self.display_object);
            view.shape.sprite.size.set(size);
            view.shape.radius.set(item.z);
            view.shape.position.set(Vector2(item.x,item.y) - size / 2.0);
        });
        Ok(())
    }
}



// ===================
// === BubbleChart ===
// ===================

/// Sample implementation of a Bubble Chart.
#[derive(Debug,Shrinkwrap)]
#[allow(missing_docs)]
pub struct BubbleChart {
    #[shrinkwrap(main_field)]
    model   : BubbleChartModel,
    network : frp::Network,
    frp     : visualization::instance::Frp,
}

#[allow(missing_docs)]
impl BubbleChart {
    pub fn definition() -> Definition {
        let path = Path::builtin("[Demo] Bubble Visualization");
        Definition::new(
            Signature::new_for_any_type(path,Format::Json),
            |scene| { Ok(Self::new(scene).into()) }
        )
    }

    pub fn new(scene:&Scene) -> Self {
        let logger         = Logger::new("bubble");
        let display_object = display::object::Instance::new(&logger);
        let views          = Rc::new(RefCell::new(vec![]));
        let network        = default();
        let frp            = visualization::instance::Frp::new(&network);
        let size           = default();
        let scene          = scene.clone_ref();
        let signature      = Signature::new_for_any_type(Path::builtin("[Demo] Bubble Chart"),
                                                         Format::Json);
        let model = BubbleChartModel{display_object,views,logger,size,scene,signature};

        BubbleChart {frp,network,model} . init()
    }

    fn init(self) -> Self {
        let network = &self.network;
        let frp     = self.frp.clone_ref();
        let model   = self.model.clone_ref();

        frp::extend! { network
            eval frp.set_size ((s) model.size.set(*s));
            eval frp.send_data ([frp,model](data) {
                if let Err(e) = model.receive_data(data) {
                    frp.data_receive_error.emit(Some(e));
                }
             });
        }
        self
    }
}

impl From<BubbleChart> for Instance {
    fn from(t:BubbleChart) -> Self {
        Self::new(&t,&t.frp,&t.network)
    }
}

impl display::Object for BubbleChart {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object.display_object()
    }
}
