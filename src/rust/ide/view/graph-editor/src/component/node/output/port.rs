
use crate::prelude::*;
use enso_frp as frp;
use ensogl::gui::component;
use ensogl::gui::component::DEPRECATED_Animation;
use ensogl::display;
use ensogl::display::shape::AnyShape;
use ensogl::display::shape::BottomHalfPlane;
use ensogl::display::shape::Circle;
use ensogl::display::shape::PixelDistance;
use ensogl::display::shape::Pixels;
use ensogl::display::shape::StyleWatch;
use ensogl::display::shape::Rect;
use ensogl::display::shape::Var;
use crate::component::node;
use ensogl::data::color;
use ensogl::display::scene::Scene;
use ensogl::display::shape::primitive::def::class::ShapeOps;



// =================
// === Constants ===
// =================

const PORT_SIZE                        : f32 = 4.0;
const PORT_SIZE_MULTIPLIER_NOT_HOVERED : f32 = 0.6;
const PORT_SIZE_MULTIPLIER_HOVERED     : f32 = 1.0;
const SEGMENT_GAP_WIDTH                : f32 = 2.0;
const HOVER_AREA_PADDING               : f32 = 20.0;
const SHOW_DELAY_DURATION_MS           : f32 = 150.0;
const HIDE_DELAY_DURATION_MS           : f32 = 150.0;

const INFINITE : f32 = 99999.0;



// =====================
// === AllPortsShape ===
// =====================

/// Generic port shape implementation. The shape is of the width of the whole node and is used as a
/// base shape for all port drawing. In case of a multi-port output, the shape is cropped from both
/// sides for each port separately. The shape looks roughly like this:

/// ```ignore
///  ╭╮                            ╭╮
///  │╰────────────────────────────╯│ ▲ height
///  ╰──────────────────────────────╯ ▼ (node_size / 2) + PORT_SIZE
///  ◄──────────────────────────────►
///   width = node_width + PORT_SIZE
/// ```
///
/// The corners are rounded with the `radius = inner_radius + port_area_size`. The shape also
/// contains an underlying hover area with a padding defined as `HOVER_AREA_PADDING`.
struct AllPortsShape {
    /// The radius of the node, not the outer port radius.
    inner_radius : Var<Pixels>,
    /// The width of the node, not the outer port area width.
    inner_width  : Var<Pixels>,
    shape        : AnyShape,
    hover        : AnyShape,
}

impl AllPortsShape {
    fn new
    ( node_canvas_width  : &Var<Pixels>
    , node_canvas_height : &Var<Pixels>
    , size_norm          : &Var<f32>
    ) -> Self {

        // === Generic Info ===

        let inner_width  = node_canvas_width  - node::PADDING.px() * 2.0;
        let inner_height = node_canvas_height - node::PADDING.px() * 2.0;
        let inner_radius = node::RADIUS.px();


        // === Main Shape ===

        let shrink           = 1.px() - 1.px() * size_norm;
        let port_area_size   = PORT_SIZE.px() * size_norm;
        let port_area_width  = &inner_width  + (&port_area_size - &shrink) * 2.0;
        let port_area_height = &inner_height + (&port_area_size - &shrink) * 2.0;
        let outer_radius     = &inner_radius + &port_area_size;
        let shape            = Rect((&port_area_width,&port_area_height));
        let shape            = shape.corners_radius(&outer_radius);
        let shape            = shape - BottomHalfPlane();
        let corner_radius    = &port_area_size / 2.0;
        let corner_offset    = &port_area_width / 2.0 - &corner_radius;
        let corner           = Circle(&corner_radius);
        let left_corner      = corner.translate_x(-&corner_offset);
        let right_corner     = corner.translate_x(&corner_offset);
        let shape            = shape + left_corner + right_corner;
        let shape            = shape.into();


        // === Hover Area ===

        let hover_width  = &inner_width + &HOVER_AREA_PADDING.px() * 2.0;
        let hover_height = &inner_height / 2.0 + &HOVER_AREA_PADDING.px();
        let hover        = Rect((&hover_width,&hover_height));
        let hover        = hover.translate_y(-hover_height/2.0);
        let hover        = hover.into();

        AllPortsShape{shape,hover,inner_radius,inner_width}
    }
}



// =================
// === PortShape ===
// =================

/// Abstraction for the `MultiPortShape` and the `SinglePortShape` allowing to
/// control animation parameters for any shape implementation.
#[allow(missing_docs)]
pub trait PortShape {
    fn set_size_multiplier(&self, grow_value:f32);
    fn set_opacity(&self, opacity:f32);
}



// =======================
// === SinglePortShape ===
// =======================

pub use single_port_area::Shape as SinglePortShape;

/// A single port shape implementation. In contrast to `MultiPortShape`, this produces a much faster
/// shader code.
mod single_port_area {
    use super::*;
    use ensogl::display::shape::*;

    ensogl::define_shape_system! {
        (style:Style, grow:f32, opacity:f32, color_rgb:Vector3<f32>) {
            let overall_width  = Var::<Pixels>::from("input_size.x");
            let overall_height = Var::<Pixels>::from("input_size.y");
            let ports          = AllPortsShape::new(&overall_width,&overall_height,&grow);
            let color          = Var::<color::Rgba>::from("srgba(input_color_rgb,input_opacity)");
            let shape          = ports.shape.fill(color);
            let hover          = ports.hover.fill(color::Rgba::new(1.0,0.0,0.0,0.3));
            // let hover          = ports.hover.fill(color::Rgba::almost_transparent());
            (shape + hover).into()
        }
    }

    impl PortShape for Shape {
        fn set_size_multiplier(&self, grow_value:f32) {
            self.grow.set(grow_value)
        }

        fn set_opacity(&self, opacity:f32) {
            self.opacity.set(opacity)
        }
    }
}



// ========================
// === Multi Port Shape ===
// ========================

pub use multi_port_area::Shape as MultiPortShape;

/// Implements the shape for a segment of the OutputPort with multiple output ports.
mod multi_port_area {
    use super::*;
    use ensogl::display::shape::*;
    use std::f32::consts::PI;

    /// Compute the angle perpendicular to the shape border.
    fn compute_border_perpendicular_angle
    (shape_border_length:&Var<f32>, corner_segment_length:&Var<f32>, position:&Var<f32>)
    -> Var<f32> {
        // Here we use a trick to use a pseudo-boolean float that is either 0 or 1 to multiply a
        // value that should be returned, iff it's case is true. That way we can add return values
        // of different "branches" of which exactly one will be non-zero.

        // Transform position to be centered in the shape.
        // The `distance`s here describe the distance along the shape border, so not straight line
        // x coordinate, but the length of the path along the shape.
        let center_distance          = position - shape_border_length  / 2.0;
        let center_distance_absolute = center_distance.abs();
        let center_distance_sign     = center_distance.signum();

        let end                = shape_border_length / 2.0;
        let center_segment_end = &end - corner_segment_length;
        let default_rotation   = Var::<f32>::from(90.0_f32.to_radians());

        // Case 1: The center segment, always zero, so not needed, due to clamping.
        // Case 2: The right circle segment.
        let relative_position     = (center_distance_absolute - center_segment_end)
                                    / corner_segment_length;
        let relative_position     = relative_position.clamp(0.0.into(),1.0.into());
        let corner_base_rotation  = (-90.0_f32).to_radians();
        let corner_rotation_delta = relative_position * corner_base_rotation;

        corner_rotation_delta * center_distance_sign + default_rotation
    }

    /// Returns the x position of the crop plane as a fraction of the base shapes center segment.
    /// To get the actual x position of the plane, multiply this value by the length of the center
    /// segment and apply the appropriate x-offset.
    ///
    /// * `shape_border_length`      should be the length of the shapes border path.
    /// * `corner_segment_length`    should be the quarter circumference of the circles on the
    ///                              sides of base shape.
    /// * `position_on_path`         should be the position along the shape border
    ///                              (not the pure x-coordinate).
    fn calculate_crop_plane_position_relative_to_center_segment
    (shape_border_length:&Var<f32>, corner_segment_length:&Var<f32>, position_on_path:&Var<f32>)
    -> Var<f32> {
        let middle_segment_start_point = corner_segment_length;
        let middle_segment_end_point   = shape_border_length - corner_segment_length;
        // Case 1: The left circle, always 0, achieved through clamping.
        // Case 2: The middle segment.
        let middle_segment_plane_position_x = (position_on_path - middle_segment_start_point)
            / (&middle_segment_end_point - middle_segment_start_point);
        // Case 3: The right circle, always 1, achieved through clamping.
        middle_segment_plane_position_x.clamp(0.0.into(), 1.0.into())
    }

    /// Compute the crop plane at the location of the given port index. Also takes into account an
    /// `position_offset` that is given as an offset along the shape boundary.
    ///
    /// The crop plane is a `HalfPlane` that is perpendicular to the border of the shape and can be
    /// used to crop the shape at the specified port index.
    fn compute_crop_plane
    ( index           : &Var<f32>
    , port_num        : &Var<f32>
    , width           : &Var<f32>
    , corner_radius   : &Var<f32>
    , position_offset : &Var<f32>
    ) -> AnyShape {
        let corner_circumference  = corner_radius * 2.0 * PI;
        let corner_segment_length = &corner_circumference * 0.25;
        let center_segment_length = width - corner_radius * 2.0;
        let shape_border_length   = &center_segment_length + &corner_segment_length * 2.0;

        let position_relative = index / port_num;
        let crop_segment_pos  = &position_relative * &shape_border_length + position_offset;

        let crop_plane_pos_relative = calculate_crop_plane_position_relative_to_center_segment
            (&shape_border_length,&corner_segment_length,&crop_segment_pos);
        let crop_plane_pos          = crop_plane_pos_relative * &center_segment_length;
        let crop_plane_pos          = crop_plane_pos + corner_radius;

        let plane_rotation_angle = compute_border_perpendicular_angle
            (&shape_border_length,&corner_segment_length,&crop_segment_pos);
        let plane_shape_offset  = Var::<Pixels>::from(&crop_plane_pos - width * 0.5);

        let crop_shape = HalfPlane();
        let crop_shape = crop_shape.rotate(plane_rotation_angle);
        let crop_shape = crop_shape.translate_x(plane_shape_offset);

        crop_shape.into()
    }

    ensogl::define_shape_system! {
        ( style         : Style
        , grow          : f32
        , index         : f32
        , port_num      : f32
        , opacity       : f32
        , padding_left  : f32
        , padding_right : f32
        , color_rgb     : Vector3<f32>
        ) {
            let overall_width  = Var::<Pixels>::from("input_size.x");
            let overall_height = Var::<Pixels>::from("input_size.y");
            let ports          = AllPortsShape::new(&overall_width,&overall_height,&grow);

            let inner_radius = Var::<f32>::from(ports.inner_radius);
            let inner_width  = Var::<f32>::from(ports.inner_width);

            let left_shape_crop = compute_crop_plane
                (&index,&port_num,&inner_width,&inner_radius,&0.0.into());
            let right_shape_crop = compute_crop_plane
                (&(Var::<f32>::from(1.0) + &index),&port_num,&inner_width,&inner_radius,&0.0.into());

            let hover_area = ports.hover.difference(&left_shape_crop);
            let hover_area = hover_area.intersection(&right_shape_crop);
            let hover_area = hover_area.fill(color::Rgba::new(1.0,0.0,0.0,0.3));
            // let hover_area = hover_area.fill(color::Rgba::almost_transparent());

            let padding_left  = Var::<Pixels>::from(padding_left);
            let padding_right = Var::<Pixels>::from(padding_right);

            let left_shape_crop  = left_shape_crop.grow(padding_left);
            let right_shape_crop = right_shape_crop.grow(padding_right);

            let port_area = ports.shape.difference(&left_shape_crop);
            let port_area = port_area.intersection(&right_shape_crop);

            let color     = Var::<color::Rgba>::from("srgba(input_color_rgb,input_opacity)");
            let port_area = port_area.fill(color);

            (port_area + hover_area).into()
        }
    }

    impl PortShape for Shape {
        fn set_size_multiplier(&self, grow_value:f32) {
            self.grow.set(grow_value)
        }

        fn set_opacity(&self, opacity:f32) {
            self.opacity.set(opacity)
        }
    }
}



// ==================
// === Shape View ===
// ==================

/// Abstraction over `ShapeView<SinglePortShape>` and `ShapeView<MultiPortShape>`.
#[derive(Clone,CloneRef,Debug)]
pub enum PortShapeView {
    Single (SinglePortShapeView),
    Multi  (MultiPortShapeView),
}

#[derive(Clone,CloneRef,Debug)]
pub struct SinglePortShapeView {
    shape : component::ShapeView<SinglePortShape>
}

#[derive(Clone,CloneRef,Debug)]
pub struct MultiPortShapeView {
    display_object : display::object::Instance,
    shapes         : Rc<Vec<component::ShapeView<MultiPortShape>>>,
}

impl SinglePortShapeView {
    pub fn new(logger:&Logger, scene:&Scene) -> Self {
        Self {shape : component::ShapeView::new(&logger,&scene) }
    }
}

impl MultiPortShapeView {
    pub fn new(logger:&Logger, scene:&Scene) -> Self {
        let display_object = display::object::Instance::new(logger);
        let shapes         = default();
        Self {display_object,shapes}
    }
}

impl PortShapeView {
    /// Constructor. If the port count is 0, we will still show a single port.
    fn new(number_of_ports: usize, logger: &Logger, scene: &Scene) -> Self {
        if number_of_ports <= 1 {
            Self::Single(SinglePortShapeView::new(logger, scene))
        } else {
            Self::Multi(MultiPortShapeView::new(logger, scene))
            // let display_object  = display::object::Instance::new(logger);
            // let mut shapes       = Vec::default();
            // let number_of_ports = number_of_ports as usize;
            // shapes.resize_with(number_of_ports,|| component::ShapeView::new(&logger,&scene));
            // shapes.iter().for_each(|shape| shape.display_object().set_parent(&display_object));
            // Self::Multi {display_object,shapes}
        }
    }
//
//     /// Set up the frp for all ports.
//     fn init_frp(&self, network:&frp::Network, port_frp:port::Frp) {
//         match self {
//             Self::Single {shape}     => port::init_port_frp(&shape,PortId::new(0),port_frp,network),
//             Self::Multi  {shapes,..} => {
//                 shapes.iter().enumerate().for_each(|(index,shape)| {
//                     port::init_port_frp(&shape,PortId::new(index),port_frp.clone_ref(),network)
//                 })
//             }
//         }
//     }
//
//     /// Resize all the port output shapes to fit the new layout requirements for thr given
//     /// parameters.
//     fn update_shape_layout_based_on_size_and_gap(&self, size:Vector2<f32>, gap_width:f32) {
//         match self {
//             Self::Single{shape} => {
//                 let shape = &shape.shape;
//                 shape.sprite.size.set(size);
//             }
//             Self::Multi{shapes,..} => {
//                 let port_num = shapes.len() as f32;
//                 for (index,shape) in shapes.iter().enumerate(){
//                     let shape = &shape.shape;
//                     shape.sprite.size.set(size);
//                     shape.index.set(index as f32);
//                     shape.port_num.set(port_num);
//                     shape.padding_left.set(gap_width * 0.5);
//                     shape.padding_right.set(-gap_width * 0.5);
//                 }
//                 shapes[0]              .shape.padding_left.set(-INFINITE);
//                 shapes[shapes.len() - 1].shape.padding_right.set(INFINITE);
//             }
//         }
//     }
//
//     fn set_color(&self, port_id:PortId, color:impl Into<color::Rgba>) {
//         let color = color.into();
//         let color = Vector3::<f32>::new(color.red,color.green,color.blue);
//         match self {
//             Self::Single{shape} => {
//                 if port_id.index == 0 {
//                     let shape = &shape.shape;
//                     shape.color_rgb.set(color)
//                 }
//             }
//             Self::Multi{shapes,..} => {
//                 if let Some(shape) = shapes.get(port_id.index) {
//                     let shape = &shape.shape;
//                     shape.color_rgb.set(color)
//                 }
//             }
//         }
//     }
// }
}

impl display::Object for PortShapeView {
    fn display_object(&self) -> &display::object::Instance {
        match self {
            Self::Single (view) => view.shape.display_object(),
            Self::Multi  (view) => &view.display_object,
        }
    }
}



// ===============
// === PortId  ===
// ===============

/// Id of a specific port inside of `OutPutPortsData`.
#[derive(Clone,Copy,Default,Debug,Eq,Hash,PartialEq)]
pub struct PortId {
    index: usize,
}

impl PortId {
    fn new(index:usize) -> Self {
        Self{index}
    }
}


// =================
// === Port Frp  ===
// =================

ensogl::define_endpoints! {
    Input {
        mouse_over (PortId),
        mouse_out  (PortId),
        mouse_down (PortId),
        hide       (),
        activate_and_highlight_selected (PortId) // FIXME : naming
    }

    Output {
    }
}


/// Set up the FRP system for a ShapeView of a shape that implements the PortShapeApi.
///
/// This allows us to use the same setup code for bot the `MultiPortShape` and the
/// `SinglePortShape`.
pub fn init_port_frp<Shape: display::shape::system::Shape + PortShape + CloneRef + 'static>
(view:&component::ShapeView<Shape>, port_id:PortId, frp:Frp, network:&frp::Network) {
    let shape        = &view.shape;
    let port_size    = DEPRECATED_Animation::<f32>::new(&network);
    let port_opacity = DEPRECATED_Animation::<f32>::new(&network);
    let frp          = &frp.input;

    frp::extend! { network

        // === Mouse Event Handling ===

        eval_ view.events.mouse_over (frp.mouse_over.emit(port_id));
        eval_ view.events.mouse_out  (frp.mouse_out.emit(port_id));
        eval_ view.events.mouse_down (frp.mouse_down.emit(port_id));


        // === Animation Handling ===

        eval port_size.value    ((size) shape.set_size_multiplier(*size));
        eval port_opacity.value ((size) shape.set_opacity(*size));


        // === Visibility and Highlight Handling ===

         eval_ frp.hide ([port_size,port_opacity]{
             port_size.set_target_value(0.0);
             port_opacity.set_target_value(0.0);
         });

        // Through the provided ID we can infer whether this port should be highlighted.
        is_selected      <- frp.activate_and_highlight_selected.map(move |id| *id == port_id);
        show_normal      <- frp.activate_and_highlight_selected.gate_not(&is_selected);
        show_highlighted <- frp.activate_and_highlight_selected.gate(&is_selected);

        eval_ show_highlighted ([port_opacity,port_size]{
            port_opacity.set_target_value(1.0);
            port_size.set_target_value(PORT_SIZE_MULTIPLIER_HOVERED);
        });

        eval_ show_normal ([port_opacity,port_size] {
            port_opacity.set_target_value(0.5);
            port_size.set_target_value(PORT_SIZE_MULTIPLIER_NOT_HOVERED);
        });
    }
}

#[derive(Clone,Debug,Default)]
pub struct Model {
    frp   : Option<Frp>,
    shape : Option<PortShapeView>,
    pub index           : usize,
    pub length          : usize,
}

impl Model {
    pub fn init_shape
    (&mut self, logger:impl AnyLogger, scene:&Scene) -> PortShapeView {
        let logger_name = format!("port({},{})",self.index,self.length);
        let logger      = Logger::sub(logger,logger_name);
        let shape       = PortShapeView::new(1,&logger,scene);
        self.shape      = Some(shape);
        self.shape.as_ref().unwrap().clone_ref()
    }
}