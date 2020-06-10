//! Implements the segmented output port area.
use crate::prelude::*;

use ensogl::display::traits::*;

use enso_frp as frp;
use enso_frp;
use ensogl::data::color;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Sprite;
use ensogl::display::scene::Scene;
use ensogl::display;
use ensogl::gui::component::Animation;
use ensogl::gui::component::Tween;
use ensogl::gui::component;

use crate::node;


// =================
// === Constants ===
// =================
// TODO: These values should be in some IDE configuration.

const BASE_SIZE           : f32 = 0.5;
const HIGHLIGHT_SIZE      : f32 = 1.0;
const SEGMENT_GAP_WIDTH   : f32 = 2.0;

const SHOW_DELAY_DURATION : f32 = 150.0;
const HIDE_DELAY_DURATION : f32 = 25.0;



// ==============
// === Shapes ===
// ==============

/// The port area shape is based on a single shape that gets offset to show a different slice for
/// each segment. Each shapes represents a window of the underlying shape.
pub mod port_area {
    use super::*;
    use ensogl::display::shape::*;

    ensogl::define_shape_system! {
        (style:Style, grow:f32, shape_width:f32, offset_x:f32, padding:f32, opacity:f32) {
            let width  : Var<Distance<Pixels>> = shape_width.into();
            let height : Var<Distance<Pixels>> = "input_size.y".into();
            let width  = width  - node::NODE_SHAPE_PADDING.px() * 2.0;
            let height = height - node::NODE_SHAPE_PADDING.px() * 2.0;

            let hover_area_size   = 20.0.px();
            let hover_area_width  = &width  + &hover_area_size * 2.0;
            let hover_area_height = &height / 2.0 + &hover_area_size;
            let hover_area        = Rect((&hover_area_width,&hover_area_height));
            let hover_area        = hover_area.translate_y(-hover_area_height/2.0);
            let hover_area        = hover_area.fill(color::Rgba::new(0.0,0.0,0.0,0.000_001));

            let shrink           = 1.px() - 1.px() * &grow;
            let radius           = node::NODE_SHAPE_RADIUS.px();
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

            // Crop the sides of the visible area to show a gap between segments.
            let overall_width     = Var::<Distance<Pixels>>::from("input_size.x");
            let padding           = Var::<Distance<Pixels>>::from(&padding * 2.0);
            let crop_window_width = &overall_width - &padding;
            let crop_window       = Rect((&crop_window_width,&height));
            let crop_window       = crop_window.translate_y(-height * 0.5);
            let port_area_cropped = crop_window.intersection(port_area_aligned);

            // FIXME: Use colour from style and apply transparency there.
            let color             = Var::<color::Rgba>::from("srgba(0.25,0.58,0.91,input_opacity)");
            let port_area_colored = port_area_cropped.fill(color);

            (port_area_colored + hover_area).into()
        }
    }
}



// ===========
// === Frp ===
// ===========

/// Id of a specific port inside of `OutPutPortsData`.
type PortId = usize;

/// Frp API of the `OutPutPorts`.
#[derive(Clone,CloneRef,Debug)]
pub struct Frp {
    /// Update the size of the `OutPutPorts`. Should match the size of the parent node for visual correctness.
    pub set_size        : frp::Source<V2<f32>>,
    /// Emitted whenever one of the ports receives a `MouseDown` event. The `PortId` indicates the source port.
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
        Self{set_size,port_mouse_down,on_port_mouse_down}
    }
}



// =======================
// === OutPutPortsData ===
// =======================

/// Internal data of the `OutPutPorts`.
#[derive(Debug)]
pub struct OutputPortsData {
    display_object : display::object::Instance,
    logger         : Logger,
    size           : Cell<Vector2<f32>>,
    gap_width      : Cell<f32>,
    ports          : RefCell<Vec<component::ShapeView<port_area::Shape>>>,
}

impl OutputPortsData {

    fn new(scene:Scene, number_of_ports:u32) -> Self {
        let logger         = Logger::new("OutPutPorts");
        let display_object = display::object::Instance::new(&logger);
        let size           = Cell::new(Vector2::zero());
        let gap_width      = Cell::new(SEGMENT_GAP_WIDTH);

        let mut ports      = Vec::default();
        ports.resize_with(number_of_ports as usize,|| component::ShapeView::new(&logger,&scene));
        let ports          = RefCell::new(ports);

        OutputPortsData {display_object,logger,size,ports,gap_width}.init()
    }

    fn init(self) -> Self {
        self.update_shape_layout_based_on_size();
        self
    }

    fn update_shape_layout_based_on_size(&self) {
        let port_num   = self.ports.borrow().len() as f32;
        let width      = self.size.get().x;
        let height     = self.size.get().y;
        let port_width = (width - node::NODE_SHAPE_PADDING) / port_num;
        let port_size  = Vector2::new(port_width, height);
        let gap_width  = self.gap_width.get();

        // Align shapes along width.
        let x_start = -width / 2.0 + node::NODE_SHAPE_PADDING;
        let x_delta = port_width;
        for (index, view) in self.ports.borrow().iter().enumerate(){
            view.display_object().set_parent(&self.display_object);

            let pos_x = x_start + x_delta * index as f32;
            let pos_y = 0.0;
            let pos   = Vector2::new(pos_x,pos_y);
            view.set_position_xy(pos);

            let shape = &view.shape;
            shape.sprite.size().set(port_size);
            shape.shape_width.set(width);
            shape.padding.set(gap_width);
            shape.offset_x.set(x_delta * index as f32);
        }
    }

    fn set_size(&self, size:Vector2<f32>) {
        self.size.set(size);
        self.update_shape_layout_based_on_size();
    }
}



// ===================
// === OutPutPorts ===
// ===================

/// Implements the segmented output port area. Provides shapes that can be attached to a `Node` to
/// add an interactive area with output ports.
///
/// The `OutputPorts` facilitate the falling behaviour:
///  * when one of the output ports is hovered, after a set time, all ports are show and the hovered
///    port is highlighted.
///  * when a different port is hovered, it is highlighted immediately.
///  * when none of the ports is hovered all of the `OutputPorts` disappear. Note: there is a very
///    small delay for disappearing to allow for smooth switching between ports.
///
#[derive(Debug,Clone,CloneRef)]
pub struct OutputPorts {
    /// The FRP api of the `OutPutPorts`.
    pub frp     : Frp,
        network : frp::Network,
        data    : Rc<OutputPortsData>,
}

impl OutputPorts {
    /// Constructor.
    pub fn new(scene:&Scene, number_of_ports:u32) -> Self {
        let network = default();
        let frp     = Frp::new(&network);
        let data    = OutputPortsData::new(scene.clone_ref(), number_of_ports);
        let data    = Rc::new(data);
        OutputPorts {data,network,frp}.init()
    }

    fn init(mut self) -> Self {
        self.init_frp();
        self
    }

    fn init_frp(&mut self) {
        let network = &self.network;
        let frp     = &self.frp;
        let data    = &self.data;

        // Used to set and detect the end of the tweens. The actual value is irrelevant, only the
        // duration of the tween matters and that this value is reached after that time.
        const TWEEN_END_VALUE:f32 = 1.0;

        // Timer used to measure whether the hover has been long enough to show the ports.
        let delay_show = Tween::new(&network);
        delay_show.set_duration(SHOW_DELAY_DURATION);

        // Timer used to measure whether the mouse has been gone long enough to hide all ports.
        let delay_hide = Tween::new(&network);
        delay_hide.set_duration(HIDE_DELAY_DURATION);

        frp::extend! { network


            // === Size Change Handling == ///

            eval frp.set_size ((size) data.set_size(size.into()));


            // === Hover Event Handling == ///

            port_mouse_over           <- source::<PortId>();
            port_mouse_out            <- source::<PortId>();

            on_show_delay_finish <- source::<()>();
            on_hide_delay_finish <- source::<()>();
            is_visible           <- source::<bool>();

            mouse_over_while_inactive  <- port_mouse_over.gate_not(&is_visible).constant(());
            mouse_over_while_active    <- port_mouse_over.gate(&is_visible).constant(());

            eval mouse_over_while_inactive ([delay_show](_){
                delay_show.set_end_value(TWEEN_END_VALUE)
            });
            eval port_mouse_out ([delay_hide](_){
                delay_hide.set_end_value(TWEEN_END_VALUE)
            });

            eval delay_show.value ([on_show_delay_finish](value) {
                if *value == TWEEN_END_VALUE{on_show_delay_finish.emit(())}
            });

            eval delay_hide.value ([on_hide_delay_finish](value) {
                if *value == TWEEN_END_VALUE {on_hide_delay_finish.emit(())}
            });

            // Ports need to be visible either because we had the delay_show timer run out (that
            // means there is an active hover) or because we had a MouseOver event before the
            // delay_hide ran out (that means that we probably switched between ports).
            activate_ports <- any(mouse_over_while_active,on_show_delay_finish);
            eval_ activate_ports (is_visible.emit(true);delay_hide.stop());

            // This is provided for ports to act on their activation and will be used further down
            // in the ports initialisation code.
            activate_ports_with_selected <- port_mouse_over.sample(&is_visible);

            // This is provided for ports to hide themselves. This is used in the port
            // Initialisation code further down.
            hide_all <- on_hide_delay_finish.map(f_!(delay_show.rewind();is_visible.emit(false)));

        }

        // Init ports
        for (index,view) in data.ports.borrow().iter().enumerate() {
            let shape        = &view.shape;
            let port_size    = Animation::<f32>::new(&network);
            let port_opacity = Animation::<f32>::new(&network);

            frp::extend! { network


                // === Mouse Event Handling == ///

                eval_ view.events.mouse_over(port_mouse_over.emit(index));
                eval_ view.events.mouse_out(port_mouse_out.emit(index));
                eval_ view.events.mouse_down(frp.on_port_mouse_down.emit(index));


                 // === Animation Handling == ///

                 eval port_size.value    ((size) shape.grow.set(*size));
                 eval port_opacity.value ((size) shape.opacity.set(*size));


                // === Visibility and Highlight Handling == ///

                 def _hide_all = hide_all.map(f_!([port_size,port_opacity]{
                     port_size.set_target_value(0.0);
                     port_opacity.set_target_value(0.0);
                 }));

                // Through the provided ID we can infer whether this port should be highlighted.
                is_selected      <- activate_ports_with_selected.map(move |id| *id == index);
                show_normal      <- activate_ports_with_selected.gate_not(&is_selected);
                show_highlighted <- activate_ports_with_selected.gate(&is_selected);

                eval_ show_highlighted ([port_opacity,port_size]{
                    port_opacity.set_target_value(1.0);
                    port_size.set_target_value(HIGHLIGHT_SIZE);
                });

                eval_ show_normal ([port_opacity,port_size]
                    port_opacity.set_target_value(0.5);
                    port_size.set_target_value(BASE_SIZE);
                );
            }
        }

        // FIXME this is a hack to ensure the ports are invisible at startup.
        // Right now we get some of FRP mouse events on startup that leave the
        // ports visible by default.
        // Once that is fixed, remove this line.
        on_hide_delay_finish.emit(());
    }

    // TODO: Implement proper sorting and remove.
    /// Hack function used to register the elements for the sorting purposes. To be removed.
    pub(crate) fn order_hack(scene:&Scene) {
        let logger = Logger::new("hack");
        component::ShapeView::<port_area::Shape>::new(&logger,scene);
    }
}

impl display::Object for OutputPorts {
    fn display_object(&self) -> &display::object::Instance {
        &self.data.display_object
    }
}
