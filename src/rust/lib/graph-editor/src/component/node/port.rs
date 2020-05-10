//! Definition of the Port component.

use crate::prelude::*;

//use crate::component::node::port::Registry;

use enso_frp;
use enso_frp as frp;
use ensogl::data::color;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Sprite;
use ensogl::display::scene::Scene;
use ensogl::display::scene::ShapeRegistry;
use ensogl::display::shape::*;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::component::animation;
use ensogl::gui::component;
use ensogl::display::shape::text::glyph::font::FontRegistry;
use ensogl::display::shape::text::glyph::system::GlyphSystem;

use crate::component::cursor;


// ============
// === Port ===
// ============

/// Canvas node shape definition.
pub mod shape {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style, hover:f32) {
            let width  : Var<Distance<Pixels>> = "input_size.x".into();
            let height : Var<Distance<Pixels>> = "input_size.y".into();
            let radius = 6.px();
            let shape  = Rect((&width,&height)).corners_radius(radius);
            // let color  : Var<color::Rgba> = "srgba(1.0,1.0,1.0,0.00001 + 0.1*input_hover)".into();
            let color  : Var<color::Rgba> = "srgba(1.0,1.0,1.0,0.00001)".into();
            let shape  = shape.fill(color);
            shape.into()
        }
    }
}


//
//// ==============
//// === Events ===
//// ==============
//
///// Port events.
//#[derive(Clone,CloneRef,Debug)]
//#[allow(missing_docs)]
//pub struct Events {
//    pub network    : frp::Network,
//    pub select     : frp::Source,
//    pub deselect   : frp::Source,
//}
//
//
//
//// ============
//// === Port ===
//// ============
//
///// Port definition.
//#[derive(AsRef,Clone,CloneRef,Debug,Deref)]
//pub struct Port {
//    data : Rc<PortData>,
//}
//
//impl AsRef<Port> for Port {
//    fn as_ref(&self) -> &Self {
//        self
//    }
//}
//
///// Weak version of `Port`.
//#[derive(Clone,CloneRef,Debug)]
//pub struct WeakPort {
//    data : Weak<PortData>
//}
//
//impl WeakElement for WeakPort {
//    type Strong = Port;
//
//    fn new(view: &Self::Strong) -> Self {
//        view.downgrade()
//    }
//
//    fn view(&self) -> Option<Self::Strong> {
//        self.upgrade()
//    }
//}
//
/////// Shape view for Port.
////#[derive(Debug,Clone,CloneRef,Copy)]
////pub struct PortView {}
////impl component::ShapeViewDefinition for PortView {
////    type Shape = shape::Shape;
////}
//
///// Internal data of `Port`
//#[derive(Debug)]
//#[allow(missing_docs)]
//pub struct PortData {
//    pub object : display::object::Instance,
//    pub logger : Logger,
//    pub events : Events,
//    pub view   : component::ShapeView<shape::Shape>,
//}
//
//impl Port {
//    /// Constructor.
//    pub fn new(scene:&Scene) -> Self {
//        frp::new_network! { node_network
//            def label    = source::<String> ();
//            def select   = source::<()>     ();
//            def deselect = source::<()>     ();
//        }
//        let network = node_network;
//        let logger  = Logger::new("node");
//        let view    = component::ShapeView::<shape::Shape>::new(&logger,scene);
//        let events  = Events {network,select,deselect};
//        let object  = display::object::Instance::new(&logger);
//        object.add_child(&view.display_object);
//
//        let width = 34.5;
//        let height = 18.0;
//
//        view.shape.sprite.size().set(Vector2::new(width,height));
//        view.mod_position(|t| t.x += width/2.0 + 81.5);
//        view.mod_position(|t| t.y += height/2.0 + 5.0);
//        let data    = Rc::new(PortData {object,logger,events,view});
//
//        Self {data}
//    }
//}
//
//impl StrongRef for Port {
//    type WeakRef = WeakPort;
//    fn downgrade(&self) -> WeakPort {
//        WeakPort {data:Rc::downgrade(&self.data)}
//    }
//}
//
//impl WeakRef for WeakPort {
//    type StrongRef = Port;
//    fn upgrade(&self) -> Option<Port> {
//        self.data.upgrade().map(|data| Port{data})
//    }
//}
//
//impl display::Object for Port {
//    fn display_object(&self) -> &display::object::Instance {
//        &self.object
//    }
//}
//
//impl display::WeakObject for WeakPort {
//    fn try_display_object(&self) -> Option<display::object::Instance> {
//        self.upgrade().map(|ref t| t.display_object().clone_ref())
//    }
//}

pub fn sort_hack(scene:&Scene) {
    let logger = Logger::new("hack");
    component::ShapeView::<shape::Shape>::new(&logger,scene);
}


#[derive(Debug,Clone,CloneRef)]
pub struct Events {
    pub network     : frp::Network,
    pub cursor_mode : frp::Stream<cursor::Mode>,
    pub press  : frp::Stream<span_tree::Crumbs>,
}



// ===============
// === Manager ===
// ===============

#[derive(Clone,CloneRef,Debug)]
pub struct Manager {
    logger         : Logger,
    display_object : display::object::Instance,
    pub frp         : Events,
    scene          : Scene,
    ports          : Rc<RefCell<Vec<component::ShapeView<shape::Shape>>>>,
}

impl Manager {
    pub fn new(logger:&Logger, scene:&Scene) -> Self {
        let logger         = logger.sub("port_manager");
        let display_object = display::object::Instance::new(&logger);
        let scene          = scene.clone_ref();


        frp::new_network! { network
            def cursor_mode = gather::<cursor::Mode>();
            def press  = source::<span_tree::Crumbs>();
        }


        let span_tree = span_tree_mock();

        let mut to_visit = vec![span_tree.root_ref()];
        let mut ports    = vec![];

        let mut off = 0.0;
        loop {
            match to_visit.pop() {
                None => break,
                Some(node) => {
                    let span          = node.span();
                    let contains_root = span.index.value == 0;
                    let skip          = node.kind.is_empty() || contains_root;
                    if !skip {
                        let port   = component::ShapeView::<shape::Shape>::new(&logger,&scene);
                        let unit   = 7.2246094;
                        let width  = unit * span.size.value as f32;
                        let width2  = width + 4.0;
                        let node_height = 28.0;
                        let height = 18.0;
                        port.shape.sprite.size().set(Vector2::new(width2,node_height));
                        let x = width/2.0 + unit * span.index.value as f32;
                        port.mod_position(|t| t.x = x);
                        port.mod_position(|t| t.y = off);
                        display_object.add_child(&port);

//                        let network = &port.events.network;
                        let hover   = &port.shape.hover;
                        let crumbs  = node.crumbs.clone();
                        frp::extend! { network
                            def _foo = port.events.mouse_over . map(f_!((hover) { hover.set(1.0); }));
                            def _foo = port.events.mouse_out  . map(f_!((hover) { hover.set(0.0); }));

                            def out  = port.events.mouse_out.constant(cursor::Mode::Normal);
                            def over = port.events.mouse_over.constant(cursor::Mode::highlight(&port,Vector2::new(x,0.0),Vector2::new(width2,height)));
                            cursor_mode.attach(&over);
                            cursor_mode.attach(&out);

                            def _press = port.events.mouse_down.map(f_!((press) {
                                press.emit(&crumbs);
                            }));
                        }
                        ports.push(port);
                    }

                    to_visit.extend(node.children_iter());
//                    off -= 3.0;

                }
            }
        }


        let ports = Rc::new(RefCell::new(ports));

        let cursor_mode = cursor_mode.into();
        let press  = press.into();
        let frp = Events {network,cursor_mode,press};

        Self {logger,display_object,frp,ports,scene}
    }
}

impl display::Object for Manager {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}



// =============
// === Mocks ===
// =============

use ast::crumbs::PatternMatchCrumb::*;
use ast::crumbs::*;
use span_tree::traits::*;


pub fn span_tree_mock() -> span_tree::SpanTree {
    let pattern_cr = vec![Seq { right: false }, Or, Or, Build];
    let val        = ast::crumbs::SegmentMatchCrumb::Body {val:pattern_cr};
    let parens_cr  = ast::crumbs::MatchCrumb::Segs {val,index:0};
    span_tree::builder::TreeBuilder::new(36)
        .add_child(0,14,span_tree::node::Kind::Chained,PrefixCrumb::Func)
        .add_leaf(0,9,span_tree::node::Kind::Operation,PrefixCrumb::Func)
        .add_empty_child(10,span_tree::node::InsertType::BeforeTarget)
        .add_leaf(10,4,span_tree::node::Kind::Target {removable:true},PrefixCrumb::Arg)
        .add_empty_child(14,span_tree::node::InsertType::Append)
        .done()
        .add_child(15,21,span_tree::node::Kind::Argument {removable:true},PrefixCrumb::Arg)
        .add_child(1,19,span_tree::node::Kind::Argument {removable:false},parens_cr)
        .add_leaf(0,12,span_tree::node::Kind::Operation,PrefixCrumb::Func)
        .add_empty_child(13,span_tree::node::InsertType::BeforeTarget)
        .add_leaf(13,6,span_tree::node::Kind::Target {removable:false},PrefixCrumb::Arg)
        .add_empty_child(19,span_tree::node::InsertType::Append)
        .done()
        .done()
        .add_empty_child(36,span_tree::node::InsertType::Append)
        .build()
}