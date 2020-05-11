//! Examples of defining visualisation in Rust using web_sys or ensogl.
use crate::prelude::*;

use crate::component::visualization::*;

use ensogl::display::DomScene;
use ensogl::display::DomSymbol;
use ensogl::display::layout::alignment;
use ensogl::display::scene::Scene;
use ensogl::display::scene::ShapeRegistry;
use ensogl::display;
use ensogl::gui::component;
use ensogl::system::web;
use std::rc::Rc;
use web::StyleSetter;



// =======================
// === HtmlBubbleChart ===
// =======================

/// Sample implementation of a Bubble Chart using `web_sys` to build SVG output.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub struct HtmlBubbleChart {
    pub content : DomSymbol,
    pub frp     : DataRendererFrp,
}

impl DataRenderer for HtmlBubbleChart {

    fn set_data(&self, data:Data) ->  Result<(),DataError> {
        let mut svg_inner = String::new();

        let data_inner: Rc<Vec<Vector2<f32>>> = data.as_binary()?;
        for pos in data_inner.iter() {
            svg_inner.push_str(
                &format!(
                    r#"<circle style="fill: #69b3a2" stroke="black" cx={} cy={} r=10.0></circle>"#,
                    pos.x, pos.y
                )
            )
        }
        self.content.dom().set_inner_html(
            &format!(r#"<svg>{}</svg>"#, svg_inner)
        );
        Ok(())
    }

    fn set_size(&self, size:Vector2<f32>) {
        self.content.set_size(size);
    }

    fn frp(&self) -> &DataRendererFrp {
        unimplemented!()
    }
}

#[allow(missing_docs)]
impl HtmlBubbleChart {
    pub fn new() -> Self {
        let div = web::create_div();
        div.set_style_or_panic("width","100px");
        div.set_style_or_panic("height","100px");

        let content = web::create_element("div");
        content.set_inner_html("<svg></svg>");
        content.set_attribute("width","100%").unwrap();
        content.set_attribute("height","100%").unwrap();

        div.append_child(&content).unwrap();

        let r          = 102_u8;
        let g          = 153_u8;
        let b          = 194_u8;
        let color      = iformat!("rgb({r},{g},{b})");
        div.set_style_or_panic("background-color",color);

        let content = DomSymbol::new(&div);
        content.dom().set_attribute("id","vis").unwrap();
        content.dom().style().set_property("overflow","hidden").unwrap();
        content.set_size(Vector2::new(100.0, 100.0));
        content.set_position(Vector3::new(0.0, 0.0, 0.0));

        let frp = default();

        HtmlBubbleChart { content,frp }
     }

    pub fn set_dom_layer(&self, scene:&DomScene) {
        scene.manage(&self.content);
    }

}

impl Default for HtmlBubbleChart {
    fn default() -> Self {
        Self::new()
    }
}

impl display::Object for HtmlBubbleChart {
    fn display_object(&self) -> &display::object::Instance {
        &self.content.display_object()
    }
}


// ========================
// === WebglBubbleChart ===
// ========================

/// Bubble shape definition.
pub mod shape {
    use super::*;
    use ensogl::display::shape::*;
    use ensogl::display::scene::Scene;
    use ensogl::data::color::*;
    use ensogl::display::Sprite;
    use ensogl::display::Buffer;
    use ensogl::display::Attribute;

    ensogl::define_shape_system! {
        (position:Vector2<f32>,radius:f32) {
            let node = Circle(radius);
            let node = node.fill(Srgb::new(0.17,0.46,0.15));
            let node = node.translate(("input_position.x","input_position.y"));
            node.into()
        }
    }
}

/// Shape view for Bubble.
#[derive(Debug,Clone)]
#[allow(missing_copy_implementations)]
pub struct BubbleView {}
impl component::ShapeViewDefinition for BubbleView {
    type Shape = shape::Shape;
    fn new(shape:&Self::Shape, _scene:&Scene, shape_registry:&ShapeRegistry) -> Self {
        shape.sprite.size().set(Vector2::new(100.0,100.0));
        let shape_system = shape_registry.shape_system(PhantomData::<shape::Shape>);
        shape_system.shape_system.set_alignment(alignment::HorizontalAlignment::Left,
                                                alignment::VerticalAlignment::Bottom);
        Self {}
    }
}

/// Sample implementation of a Bubble Chart using `WebGl`.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct WebglBubbleChart {
    pub display_object : display::object::Instance,
        frp            : DataRendererFrp,
        views          : RefCell<Vec<component::ShapeView<BubbleView>>>,
        logger         : Logger,
        size           : Cell<Vector2<f32>>,
}

#[allow(missing_docs)]
impl WebglBubbleChart {
    pub fn new() -> Self {
        let logger         = Logger::new("bubble");
        let display_object = display::object::Instance::new(&logger);
        let views          = RefCell::new(vec![]);
        let frp            = default();
        let size           = Cell::new(Vector2::zero());

        WebglBubbleChart { display_object,views,logger,frp,size }
    }
}

impl DataRenderer for WebglBubbleChart {

    fn set_data(&self, data:Data) -> Result<(),DataError> {
        let data_inner: Rc<Vec<Vector3<f32>>> = data.as_binary()?;

        // Avoid re-creating views, if we have already created some before.
        let mut views = self.views.borrow_mut();
        views.resize_with(data_inner.len(),|| component::ShapeView::new(&self.logger));

        views.iter().zip(data_inner.iter()).for_each(|(view,item)| {
            view.display_object.set_parent(&self.display_object);
            if let Some(t) = view.data.borrow().as_ref() {
                t.shape.sprite.size().set(self.size.get());
                t.shape.radius.set(item.z);
                t.shape.position.set(Vector2::new(item.x,item.y));
            };
        });
        Ok(())
    }

    fn set_size(&self, size:Vector2<f32>) {
        self.size.set(size);
    }

    fn frp(&self) -> &DataRendererFrp {
        &self.frp
    }
}

impl display::Object  for WebglBubbleChart {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object.display_object()
    }
}

impl Default for WebglBubbleChart {
    fn default() -> Self {
        Self::new()
    }
}
