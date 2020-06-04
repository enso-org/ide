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

const BASE_SIZE         : f32 = 0.5;
const HIGHLIGHT_SIZE    : f32 = 1.0;
const SEGMENT_GAP_WIDTH : f32 = 2.0;



// ==============
// === Shapes ===
// ==============

/// The port area shape is based on a single shape that gets offset to show a different slice for
/// each segment. Each shapes represents a window of the underlying shape.
pub mod port_area {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style, grow:f32, shape_width:f32, offset_x:f32, padding:f32, opacity:f32) {

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

            // Move the shape so it shows the correct slice, as indicated by `offset_x`.
            let offset_x          = Var::<Distance<Pixels>>::from(offset_x);
            let offset_x          = width/2.0 - offset_x;
            let port_area_aligned = port_area.translate_x(offset_x);

            let overall_width     = Var::<Distance<Pixels>>::from("input_size.x");
            let padding           = Var::<Distance<Pixels>>::from(&padding * 2.0);
            let crop_window_width = &overall_width - &padding;
            let crop_window       = Rect((&crop_window_width,&height));
            let crop_window       = crop_window.translate_y(-height * 0.5);

            let port_area_cropped = crop_window.intersection(port_area_aligned);

            let color             = Var::<color::Rgba>::from("srgba(0.25,0.58,0.91,input_opacity)");
            let port_area_colored = port_area_cropped.fill(color);

            (port_area_colored + hover_area).into()
        }
    }
}



// ===========
// === Frp ===
// ===========

type PortId = usize;

#[derive(Clone,CloneRef,Debug)]
pub struct Frp {
    pub set_size        : frp::Source<V2<f32>>,
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
}

impl OutPutPortsData {
    fn init(mut self) -> Self {
        self.update_shapes();
        self
    }

    fn update_ports(&mut self) {
        let port_num      = self.ports.len() as f32;

        let width         = self.size.x;
        let height        = self.size.y;
        let element_width = width / port_num;
        let element_size  = Vector2::new(element_width,height);

        // Align shapes along width.
        let x_start = -width/2.0 + NODE_SHAPE_PADDING;
        let x_delta = element_width;
        for (index, view) in self.ports.iter().enumerate(){
            view.display_object().set_parent(&self.display_object);

            let pos_x = x_start + x_delta * index as f32;
            let pos_y = 0.0;
            let pos   = Vector2::new(pos_x,pos_y);
            view.set_position_xy(pos);

            let shape = &view.shape;
            shape.sprite.size().set(element_size);
            shape.shape_width.set(width);
            shape.padding.set(self.gap_width);
            shape.offset_x.set(x_delta * index as f32);
        }
    }

    fn update_shapes(&mut self) {
        self.update_ports();
    }

    fn set_size(&mut self, size:Vector2<f32>) {
        self.size = size;
        self.update_shapes();
    }
}

// ===================
// === OutPutPorts ===
// ===================

#[derive(Debug,Clone,CloneRef)]
pub struct OutputPorts {
    pub frp            : Frp,
        network        : frp::Network,
        data           : Rc<RefCell<OutPutPortsData>>,
        display_object : display::object::Instance,
}

impl OutputPorts {
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

        let data =  OutPutPortsData {display_object:display_object.clone_ref(),scene,logger,size,
                                     ports,gap_width}.init();
        let data = Rc::new(RefCell::new(data));

        OutputPorts {data,network,frp,display_object}.init()
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
            eval  frp.set_size ((size) data.borrow_mut().set_size(size.into()));

            def set_port_size      = source::<(PortId,f32)>();
            def set_port_sizes_all = source::<f32>();

            def _set_port_sizes = set_port_sizes_all.map(f!([set_port_size,data](size) {
                let port_num = data.borrow().ports.len();
                for index in 0..port_num {
                    set_port_size.emit((index,*size));
                };
            }));

            def set_port_opacity     = source::<(PortId,f32)>();
            def set_port_opacity_all = source::<f32>();

            def _set_port_opacities = set_port_opacity_all.map(f!([set_port_opacity,data](size) {
                let port_num = data.borrow().ports.len();
                for index in 0..port_num {
                    set_port_opacity.emit((index,*size));
                };
            }));
        }

        // Init ports
        {
            for (index,view) in data.borrow().ports.iter().enumerate() {
                let shape        = &view.shape;
                let port_size    = Animation::<f32>::new(&network);
                let port_opacity = Animation::<f32>::new(&network);
                frp::extend! { network
                     eval port_size.value    ((size) shape.grow.set(*size));
                     eval port_opacity.value ((size) shape.opacity.set(*size));

                    is_resize_target <- set_port_size.map(move |(id,_)| *id == index);
                    size_change      <- set_port_size.gate(&is_resize_target);
                    eval size_change (((_, size)) port_size.set_target_value(*size));

                    is_opacity_target <- set_port_opacity.map(move |(id, _)| *id==index);
                    opacity_change    <- set_port_opacity.gate(&is_opacity_target);
                    eval opacity_change (((_, opacity)) port_opacity.set_target_value(*opacity));

                    eval_ view.events.mouse_over ([port_size,set_port_sizes_all,port_opacity,
                                                  set_port_opacity_all] {
                        set_port_sizes_all.emit(BASE_SIZE);
                        set_port_opacity_all.emit(0.5);
                        port_size.set_target_value(HIGHLIGHT_SIZE);
                        port_opacity.set_target_value(1.0);
                    });

                    eval_ view.events.mouse_out ([set_port_sizes_all,set_port_opacity_all] {
                         set_port_sizes_all.emit(0.0);
                         set_port_opacity_all.emit(0.0);
                    });

                    eval_ view.events.mouse_down(frp.on_port_mouse_down.emit(index));
                }
            }
        }
    }

    // TODO: Implement proper sorting and remove.
    /// Hack function used to register the elements for the sorting purposes. To be removed.
    pub(crate) fn init_shape_order_hack(scene:&Scene) {
        let logger = Logger::new("hack");
        component::ShapeView::<port_area::Shape>::new(&logger,scene);
    }
}

impl display::Object for OutputPorts {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
