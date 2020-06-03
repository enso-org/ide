//! Implements the segmented output port area.
use crate::prelude::*;

use enso_frp as frp;
use enso_frp;
use ensogl::data::color;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Sprite;
use ensogl::display::scene::Scene;
use ensogl::display::shape::*;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::component::Animation;
use ensogl::gui::component;

use crate::node::NODE_SHAPE_PADDING;



// =================
// === Constants ===
// =================

const BASE_SIZE:f32         = 0.5;
const HIGHLIGHT_SIZE:f32    = 1.0;
const SEGMENT_GAP_WIDTH:f32 = 2.0;



// ==============
// === Shapes ===
// ==============

/// The port area shape is based on a single shape that gets offset to show a different slice for
/// each segment. Each shapes represents a window of the underlying shape.
pub mod port_area {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style, grow:f32, shape_width:f32, offset_x:f32) {
            let width  : Var<Distance<Pixels>> = shape_width.into();
            let height : Var<Distance<Pixels>> = "input_size.y".into();
            let width  = width  - NODE_SHAPE_PADDING.px() * 2.0;
            let height = height - NODE_SHAPE_PADDING.px() * 2.0;

            let hover_area_size   = 20.0.px();
            let hover_area_width  = &width  + &hover_area_size * 2.0;
            let hover_area_height = &height / 2.0 + &hover_area_size;
            let hover_area        = Rect((&hover_area_width,&hover_area_height));
            let hover_area        = hover_area.translate_y(-hover_area_height/2.0);
            let hover_area        = hover_area.fill(color::Rgba::new(0.0,0.0,0.0,0.000_001));

            let shrink           = 1.px() - 1.px() * &grow;
            let radius           = 14.px();
            let port_area_size   = 4.0.px() * &grow;
            let port_area_width  = &width  + (&port_area_size - &shrink) * 2.0;
            let port_area_height = &height + (&port_area_size - &shrink) * 2.0;
            let bottom_radius    = &radius + &port_area_size;
            let port_area        = Rect((&port_area_width,&port_area_height));
            let port_area        = port_area.corners_radius(&bottom_radius);
            let port_area        = port_area - BottomHalfPlane();
            let corner_radius    = &port_area_size / 2.0;
            let corner_offset    = &port_area_width / 2.0 - &corner_radius;
            let corner           = Circle(&corner_radius);
            let left_corner      = corner.translate_x(-&corner_offset);
            let right_corner     = corner.translate_x(&corner_offset);
            let port_area        = port_area + left_corner + right_corner;
            let port_area        = port_area.fill(color::Rgba::from(color::Lcha::new(0.6,0.5,0.76,1.0)));

            // Move the shape so it shows the correct slice, as indicated by `offset_x`.
            let offset_x          = Var::<Distance<Pixels>>::from(offset_x);
            let offset_x          = width/2.0 - offset_x;
            let port_area_aligned = port_area.translate_x(offset_x);

            (port_area_aligned + hover_area).into()
        }
    }
}

/// Defines an invisible shape that is used to catch hover events below the main port area.
mod hover_area {
    use super::*;
    ensogl::define_shape_system! {
        () {
            let width  : Var<Distance<Pixels>> = "input_size.x".into();
            let height : Var<Distance<Pixels>> = "input_size.y".into();

            let hover_area_size   = 20.0.px();
            let hover_area_width  = &width  + &hover_area_size * 2.0;
            let hover_area_height = &height / 2.0 + &hover_area_size;
            let hover_area        = Rect((&hover_area_width,&hover_area_height));
            let hover_area        = hover_area.fill(color::Rgba::new(0.0,0.0,0.0,0.000_001));

            hover_area.into()
        }
    }
}



// ===========
// === Frp ===
// ===========

type PortId = usize;

#[derive(Clone,CloneRef,Debug)]
pub struct Frp {
    pub set_size        : frp::Source<Option<Vector2<f32>>>,
    pub port_mouse_down : frp::Stream<PortId>,

    on_port_mouse_down  : frp::Source<PortId>,
}

impl Frp {
    fn new(network: &frp::Network) -> Self {
        frp::extend! { network
            def set_size           = source();
            def on_port_mouse_down = source();

            let port_mouse_down = on_port_mouse_down.clone_ref().into();
        }
        Self{set_size,on_port_mouse_down,port_mouse_down}
    }
}



// =======================
// === OutPutPortsData ===
// =======================

#[derive(Debug)]
pub struct OutPutPortsData {
    display_object : display::object::Instance,
    scene          : Scene,
    logger         : Logger,
    size           : Vector2<f32>,
    gap_width      : f32,
    ports          : Vec<component::ShapeView<port_area::Shape>>,
    hover_area     : component::ShapeView<hover_area::Shape>,
}

impl OutPutPortsData {
    fn init(mut self) -> Self {
        self.update_shapes();
        self
    }

    fn update_hover_area(&mut self) {
        let hover_size = Vector2::new(self.size.x, self.size.y);
        let hover_pos  = Vector3::new(0.0, -hover_size.y / 2.0, 0.0);
        self.hover_area.set_position(hover_pos);
        self.hover_area.shape.sprite.size().set(hover_size);
    }

    fn update_ports(&mut self) {
        let port_num      = self.ports.len() as f32;

        let gap_width     = self.gap_width;
        let width         = self.size.x;
        let width_no_gaps = width - gap_width * (port_num - 1.0) ;
        let height        = self.size.y;
        let element_width = width_no_gaps / port_num;
        let element_size  = Vector2::new(element_width, height);

        // Align shapes along width.
        let x_start = 0.0;
        let x_delta = element_width + gap_width;
        for (index, view) in self.ports.iter().enumerate(){
            let pos_x = x_start + x_delta * index as f32;
            let pos_y = 0.0;
            let pos   = Vector2::new(pos_x,pos_y);
            view.set_position_xy(pos);

            let shape = &view.shape;
            shape.sprite.size().set(element_size);
            shape.shape_width.set(width);
            shape.grow.set(BASE_SIZE);
            shape.offset_x.set(pos_x);
        }
    }

    fn update_shapes(&mut self) {
        self.update_ports();
        self.update_hover_area();
    }

    fn set_size(&mut self, size:Vector2<f32>) {
        self.size = size;
        self.update_shapes();
    }

    fn hide_ports(&self) {
        self.ports.iter().for_each(|port| port.display_object().unset_parent())
    }

    fn show_ports(&self) {
        self.ports.iter().for_each(|port| port.display_object().set_parent(&self.display_object))
    }
}

// ===================
// === OutPutPorts ===
// ===================

#[derive(Debug,Clone,CloneRef)]
pub struct OutPutPorts {
    pub frp            : Frp,
        network        : frp::Network,
        data           : Rc<RefCell<OutPutPortsData>>,
        display_object : display::object::Instance,
}

impl OutPutPorts {
    pub fn new(scene:&Scene, number_of_ports:u8) -> Self {
        let logger         = Logger::new("bubble");

        let display_object = display::object::Instance::new(&logger);
        let network        = default();
        let frp            = Frp::new(&network);

        let size           = Vector2::zero();
        let scene          = scene.clone_ref();
        let gap_width      = SEGMENT_GAP_WIDTH;

        let mut ports      = Vec::default();
        ports.resize_with(number_of_ports as usize,|| component::ShapeView::new(&logger,&scene));
        let ports = ports;

        let hover_area     = component::ShapeView::new(&logger,&scene);
        hover_area.display_object().set_parent(&display_object);

        let data =  OutPutPortsData {display_object:display_object.clone_ref(),scene,logger,size,
                                     ports,gap_width,hover_area}.init();
        let data = Rc::new(RefCell::new(data));

        OutPutPorts {data,network,frp,display_object}.init()
    }

    fn init(mut self) -> Self {
        self.init_frp();
        self
    }

    fn init_frp(&mut self) {
        let network = &self.network;
        let frp     = &self.frp;
        let data    = &self.data;

        frp::extend! { network
            eval  frp.set_size ((size) data.borrow_mut().set_size(size.unwrap_or_else(zero)));
        }

        // Init hover area
        {
            let hover_area = &data.borrow().hover_area;
            frp::extend! { network
                eval hover_area.events.mouse_over ((_) data.borrow().show_ports());
                eval hover_area.events.mouse_out  ((_) data.borrow().hide_ports());
            }
        }

        // Init ports
        {
            for (index,view) in data.borrow().ports.iter().enumerate() {
                let shape          = &view.shape;
                let data           = self.data.clone_ref();
                let port_area_size = Animation::<f32>::new(&network);
                frp::extend! { network
                    eval port_area_size.value ((size) shape.grow.set(*size));

                    eval view.events.mouse_over ([port_area_size,data](_) {
                        data.borrow().show_ports();
                        port_area_size.set_target_value(HIGHLIGHT_SIZE);
                    });

                    eval view.events.mouse_out ([port_area_size](_) {
                        port_area_size.set_target_value(BASE_SIZE);
                          data.borrow().hide_ports();
                    });

                    eval view.events.mouse_down ([frp](_) {
                        frp.on_port_mouse_down.emit(index);
                    });
                }
            }
        }
    }

    // TODO: Implement proper sorting and remove.
    /// Hack function used to register the elements for the sorting purposes. To be removed.
    pub(crate) fn init_shape_order_hack(scene:&Scene) {
        let logger = Logger::new("hack");
        component::ShapeView::<hover_area::Shape>::new(&logger,scene);
        component::ShapeView::<port_area::Shape>::new(&logger,scene);
    }
}

impl display::Object for OutPutPorts {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
