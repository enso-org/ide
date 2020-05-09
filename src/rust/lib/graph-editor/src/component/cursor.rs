//! Definition of the Cursor (known as well as mouse pointer) component.

use crate::prelude::*;

use enso_frp as frp;
use ensogl::control::callback;
use ensogl::data::color;
use ensogl::display::Buffer;
use ensogl::display::layout::alignment;
use ensogl::display::scene::{Scene,ShapeRegistry};
use ensogl::display::scene;
use ensogl::display::shape::*;
use ensogl::display::{Sprite, Attribute};
use ensogl::display;
use ensogl::gui::component::animation;
use ensogl::gui::component::animation2;
use ensogl::gui::component;
use ensogl::system::web;


#[derive(Debug,Clone)]
pub enum Mode {
    Normal,
    Cursor,
    Highlight {
        host     : display::object::Instance,
        position : Vector2<f32>,
        size     : Vector2<f32>,
    }
}

impl Mode {
    pub fn highlight<H>(host:H, position:Vector2<f32>, size:Vector2<f32>) -> Self
    where H:display::Object {
        let host = host.display_object().clone_ref();
        Self::Highlight {host,position,size}
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}



// ==================
// === CursorView ===
// ==================

/// Canvas shape definition.
pub mod shape {
    use super::*;

    ensogl::define_shape_system! {
        (position:Vector2<f32>, width:f32, height:f32, selection_size:Vector2<f32>, press:f32, radius:f32) {
            let radius = 1.px() * radius - 2.px() * &press;
            let side   = &radius * 2.0;
            let selection_width  = 1.px() * &selection_size.x() * &press;
            let selection_height = 1.px() * &selection_size.y() * &press;
            let width            = 1.px() * &width + selection_width.abs();
            let height           = 1.px() * &height + selection_height.abs();
            let cursor = Rect((width,height))
                .corners_radius(radius)
                .translate((-&selection_width/2.0, -&selection_height/2.0))
                .translate(("input_position.x","input_position.y"))
                .fill(color::Rgba::new(1.0,1.0,1.0,0.3));
            cursor.into()
        }
    }
}

///// Shape view for Cursor.
//#[derive(Clone,CloneRef,Debug)]
//#[allow(missing_docs)]
//pub struct CursorView {}
//
//impl component::ShapeViewDefinition for CursorView {
//    type Shape = shape::Shape;
//}



// ==============
// === Events ===
// ==============

/// Cursor events.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Events {
    pub network  : frp::Network,
    pub set_mode : frp::Source<Mode>,
    pub press    : frp::Source,
    pub release  : frp::Source,
}

impl Default for Events {
    fn default() -> Self {
        frp::new_network! { cursor_events
            def set_mode = source();
            def press    = source();
            def release  = source();
        }
        let network = cursor_events;
        Self {network,set_mode,press,release}
    }
}



// ==============
// === Cursor ===
// ==============

/// Cursor (mouse pointer) definition.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct Cursor {
    data : Rc<CursorData>
}

/// Weak version of `Cursor`.
#[derive(Clone,CloneRef,Debug)]
pub struct WeakCursor {
    data : Weak<CursorData>
}

/// Internal data for `Cursor`.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct CursorData {
    pub logger : Logger,
    pub events : Events,
    pub view   : component::ShapeView<shape::Shape>,
//    pub scene_view    : scene::View,
    pub resize_handle : callback::Handle,
}

impl Cursor {
    /// Constructor.
    pub fn new(scene:&Scene) -> Self {
        let logger = Logger::new("cursor");
        let view   = component::ShapeView::<shape::Shape>::new(&logger,scene);
        let events = Events::default();




        let scene_shape = scene.shape();
        let shape       = &view.shape;
        shape.sprite.size().set(Vector2::new(scene_shape.width(),scene_shape.height()));

        let resize_handle = scene.on_resize(enclose!((shape) move |scene_shape:&web::dom::ShapeData| {
            shape.sprite.size().set(Vector2::new(scene_shape.width(),scene_shape.height()));
        }));

        let shape_system = scene.shapes.shape_system(PhantomData::<shape::Shape>);
        shape_system.shape_system.set_alignment(alignment::HorizontalAlignment::Left, alignment::VerticalAlignment::Bottom);
        shape_system.shape_system.set_pointer_events(false);

        scene.views.main.remove_from_normal_layer(&shape_system.shape_system.symbol);
        scene.views.main.add_to_cursor_layer(&shape_system.shape_system.symbol);
//        let scene_view = scene.views.new();
//        scene.views.main.remove(&shape_system.shape_system.symbol);
//        scene_view.add(&shape_system.shape_system.symbol);





        let data   = CursorData {logger,events,view,resize_handle};
        let data   = Rc::new(data);

        Cursor {data} . init(scene)
    }

    fn init(self, scene:&Scene) -> Self {

        let network = &self.data.events.network;

        let view_data = self.view.shape.clone_ref();
        let press = animation(network,move |value| {
            view_data.press.set(value)
        });

        let view_data = self.view.shape.clone_ref();
        let radius = animation(network,move |value| {
            view_data.radius.set(value)
        });

        let view_data = self.view.shape.clone_ref();
        let width = animation(network,move |value| {
            view_data.width.set(value)
        });

        let view_data = self.view.shape.clone_ref();
        let height = animation(network,move |value| {
            view_data.height.set(value)
        });


        let (anim_pos_x_setter,anim_pos_x) = animation2(network);
        let (anim_pos_y_setter,anim_pos_y) = animation2(network);


        let mouse = &scene.mouse.frp;



        let view = &self.view;

        frp::extend! { network

            def anim_pos_xy = anim_pos_x.zip(&anim_pos_y);

            def _t_press = self.events.press.map(enclose!((press) move |_| {
                press.set_target_position(1.0);
            }));

            def _t_release = self.events.release.map(enclose!((press) move |_| {
                press.set_target_position(0.0);
            }));

            def fixed_position = self.events.set_mode.map(enclose!((anim_pos_x_setter,anim_pos_y_setter) move |m| {
                match m {
                    Mode::Highlight {host,position,..} => {
                        let p = host.global_position();
                        anim_pos_x_setter.set_target_position(p.x);
                        anim_pos_y_setter.set_target_position(p.y);
                        Some(p)
                    }
                    _ => None
                }
            }));

            def uses_mouse_position = fixed_position.map(|p| p.is_none());
            def mouse_position = mouse.position.gate(&uses_mouse_position);

            def _position = anim_pos_xy.map(f!((view)(p) {
                view.shape.position.set(Vector2::new(p.0,p.1));
            }));

            def _position = mouse_position.map(f!((anim_pos_x_setter,anim_pos_y_setter)(p) {
                anim_pos_x_setter.set_target_position(p.x);
                anim_pos_y_setter.set_target_position(p.y);
//                view.shape.position.set(Vector2::new(p.x,p.y));
            }));

            def _t_mode = self.events.set_mode.map(enclose!((radius,width,height) move |m| {
                let mm = match m {
                    Mode::Normal => {
                        radius.set_target_position(8.0);
                        width.set_target_position(16.0);
                        height.set_target_position(16.0);
                    }
                    Mode::Highlight {size,..} => {
                        radius.set_target_position(4.0);
                        width.set_target_position(size.x);
                        height.set_target_position(size.y);
                    }
                    _ => panic!()
                };
            }));
        }

        radius.set_target_position(8.0);
        width.set_target_position(16.0);
        height.set_target_position(16.0);

        self.events.set_mode.emit(Mode::Normal);

        self
    }

//    /// Position setter.
//    pub fn set_position(&self, pos:Vector2<f32>) {
//        self.view.shape.position.set(pos);
//    }

    /// Selection size setter.
    pub fn set_selection_size(&self, pos:Vector2<f32>) {
        self.view.shape.selection_size.set(pos);
    }
}

impl StrongRef for Cursor {
    type WeakRef = WeakCursor;
    fn downgrade(&self) -> WeakCursor {
        WeakCursor {data:Rc::downgrade(&self.data)}
    }
}

impl WeakRef for WeakCursor {
    type StrongRef = Cursor;
    fn upgrade(&self) -> Option<Cursor> {
        self.data.upgrade().map(|data| Cursor{data})
    }
}

impl display::Object for Cursor {
    fn display_object(&self) -> &display::object::Instance {
        &self.view.display_object
    }
}
