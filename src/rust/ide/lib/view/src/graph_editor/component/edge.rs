//! Definition of the Edge component.


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
use ensogl::gui::component::ShapeViewEvents;
use ensogl::gui::component;
use nalgebra::UnitComplex;
use std::cmp::Ordering;

use super::node;



fn min(a:f32,b:f32) -> f32 {
    f32::min(a,b)
}


fn max(a:f32,b:f32) -> f32 {
    f32::max(a,b)
}




// =================
// === Constants ===
// =================

const LINE_SHAPE_WIDTH   : f32 = LINE_WIDTH + 2.0 * PADDING;
const LINE_SIDE_OVERLAP  : f32 = 1.0;
const LINE_SIDES_OVERLAP : f32 = 2.0 * LINE_SIDE_OVERLAP;
const LINE_WIDTH         : f32 = 4.0;
const MOUSE_OFFSET       : f32 = 2.0;
const NODE_PADDING       : f32 = node::SHADOW_SIZE;
const PADDING            : f32 = 4.0;
const RIGHT_ANGLE        : f32 = std::f32::consts::PI / 2.0;

const SNAP_THRESHOLD     : f32 = 1.2 * LINE_SHAPE_WIDTH;

const INFINITE : f32 = 99999.0;



// ========================
// === Edge Shape Trait ===
// ========================

/// Edge shape defines the common behaviour of the sub-shapes used to create a Edge.
trait EdgeShape: ensogl::display::Object {
    fn set_hover_split_offset(&self, offset:Vector2<f32>);
    fn set_hover_split_rotation(&self, angle:f32);
    /// Snaps the coordinates given in the shape local coordinate system to the shape and returns
    /// the shape local coordinates. If no snapping is possible, returns `None`.
    fn snap_to_shape_local(&self, point:Vector2<f32>) -> Option<Vector2<f32>>;
    fn sprite(&self) -> &Sprite;

    /// Return the angle perpendicular to the shape at the given point. Defaults to zero, if not
    /// implemented.
    fn cut_angle_for_point(&self, point:Vector2<f32>) -> nalgebra::Rotation2<f32>  {
        let local = self.to_shape_coordinate_system(point);
        self.cut_angle_for_point_local(local)
    }

    fn cut_angle_for_point_local(&self, _point:Vector2<f32>) -> nalgebra::Rotation2<f32> {
        nalgebra::Rotation2::new(0.0)
    }

    /// Snaps the coordinates given in the global coordinate system to the shape and returns
    /// the global coordinates. If no snapping is possible, returns `None`.
    fn snap_to_shape(&self, point:Vector2<f32>) -> Option<Vector2<f32>> {
        let local          = self.to_shape_coordinate_system(point);
        let local_snapped  = self.snap_to_shape_local(local)?;
        let global_snapped = self.from_shape_to_global_coordinate_system(local_snapped);
        Some(global_snapped)
    }

    /// Set the hover split for this shape. The `split_data` indicates where the shape should be
    /// split and which side should be recolored.
    fn enable_hover_split(&self, split_data:SplitData) {
        let SplitData {split_position,area,cut_angle} = split_data;
        // Compute rotation in shape local coordinate system.
        let base_rotation        = self.display_object().rotation().z;
        let hover_split_rotation = cut_angle + base_rotation;
        match area {
            Area::Above => self.set_hover_split_rotation(hover_split_rotation),
            Area::Below => {
                self.set_hover_split_rotation(hover_split_rotation + 2.0 * RIGHT_ANGLE)
            },
        }
        // Compute position in shape local coordinate system.
        let offset = self.to_shape_coordinate_system(split_position);
        self.set_hover_split_offset(offset)
    }

    /// Disable the hover split on this shape.
    fn disable_hover_split(&self) {
        self.set_hover_split_offset(Vector2::new(INFINITE, INFINITE));
        self.set_hover_split_rotation(RIGHT_ANGLE);
    }

    /// Convert the given global coordinates to the shape local coordinate system.
    fn to_shape_coordinate_system(&self, point:Vector2<f32>) -> Vector2<f32> {
        let base_rotation   = self.display_object().rotation().z;
        let local_unrotated = point - self.display_object().global_position().xy();
        UnitComplex::new(-base_rotation) * local_unrotated
    }

    /// Convert the given shape local coordinate to the global coordinates system.
    fn from_shape_to_global_coordinate_system(&self, point:Vector2<f32>) -> Vector2<f32> {
        let base_rotation   = self.display_object().rotation().z;
        let local_unrotated = UnitComplex::new(base_rotation) * point;
        local_unrotated + self.display_object().global_position().xy()
    }

    /// Check whether the given shape local coordinate is within the bounds of the shape.
    fn contains_local(&self, point:Vector2<f32>) -> bool {
        let size = self.sprite().size.get();
        if point.x < -size.x / 2.0 || point.y < -size.y / 2.0 {
            return false
        }
        point.x < size.x / 2.0 && point.y < size.y / 2.0
    }

    /// Check whether the given global coordinate is within the bounds of the shape.
    fn contains(&self, point:Vector2<f32>) -> bool {
        self.contains_local(self.to_shape_coordinate_system(point))
    }
}



// ========================
// === Edge Shape Trait ===
// ========================

/// Indicates which area should be recolored.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
enum Area {
    Above,
    Below,
}

/// Holds the data required to split a shape into two parts.
///
/// The `area` indicates which side of the split will receive special coloring.
#[derive(Clone,Copy,Debug)]
struct SplitData {
    split_position : Vector2<f32>,
    area           : Area,
    cut_angle      : f32
}

impl SplitData {
    fn new(split_position:Vector2<f32>, area: Area, cut_angle:f32) -> Self {
        SplitData {split_position,area,cut_angle}
    }
}

/// `SnapData` is used as the result value of snapping operations on `MultiShapes`. It holds the
/// position and shape that a hover position was snapped to, that is the shape and the position
/// on the shape that was closest to the hover
struct SnapData<'a> {
    position     : Vector2<f32>,
    target_shape : &'a dyn EdgeShape,
}

impl<'a> SnapData<'a> {
    fn new(position: Vector2<f32>, target_shape: &'a dyn EdgeShape) -> Self {
        SnapData{position,target_shape}
    }
}

/// The MultiShape trait allows operations on a collection of `EdgeShape`.
trait MultiShape {
    /// Return the `ShapeViewEvents` of all sub-shapes.
    fn events(&self) -> Vec<ShapeViewEvents>;
    /// Return references to all `EdgeShape`s in this MultiShape.
    fn edge_shape_views(&self) -> Vec<&dyn EdgeShape>;

    /// Connect the given `ShapeViewEventsProxy` to the mouse events of all sub-shapes.
    fn register_proxy_frp(&self, network:&frp::Network, frp:&ShapeViewEventsProxy) {
        let events = self.events();
        for event in &events {
            frp::extend! { network
                eval_ event.mouse_down (frp.on_mouse_down.emit(()));
                eval_ event.mouse_over (frp.on_mouse_over.emit(()));
                eval_ event.mouse_out (frp.on_mouse_out.emit(()));
            }
        }
    }

    /// Snap the given position to our sub-shapes. Returns `None` if the given position cannot be
    /// snapped to any sub-shape (e.g., because it is too far away).
    fn snap_position_to_shape(&self, point:Vector2<f32>) -> Option<SnapData> {
        let shapes    = self.edge_shape_views();
        let snap_data = shapes.iter().filter_map(|shape|{
            let maybe_snap_position     = shape.snap_to_shape(point);
            if let Some(snap_position) = maybe_snap_position {
                // TODO The `SNAP_THRESHOLD` test is needed to avoid visual artifacts for
                // circles where sometimes we might snap to a part that is invisible instead of
                // the visible shape next to it. It might be better to find a cheaper way to avoid
                // these.
                let snap_distance = (snap_position - point).norm();
                if snap_distance < SNAP_THRESHOLD && shape.contains(point){
                    return Some(SnapData::new(snap_position,*shape))
                }
            }
            None
        });

        // Return the snap that is closest to the hover.
        snap_data
            .map(|snap_data| ((snap_data.position - point).norm(), snap_data))
            .min_by(|(distance_a, _), (distance_b, _)| {
                distance_a.partial_cmp(&distance_b).unwrap_or(Ordering::Equal)
            })
            .map(|(_, snap_data)| snap_data)
    }

    /// Try to snap the given position to the shape and compute the perpendicular cut angle. If we
    /// can find valid values, we return a `SplitData` struct. This can fail, if the hover is too
    /// far from any of our sub-shapes.
    fn get_split_data_for_hover(&self,position:Vector2<f32>,area:Area) -> Option<SplitData> {
        let maybe_snap_data = self.snap_position_to_shape(position);
        if let Some(snap_data) = maybe_snap_data {
            // TODO refactor into shape
            let base_rotation = snap_data.target_shape.display_object().rotation().z;
            let cut_angle = snap_data.target_shape.cut_angle_for_point(snap_data.position).angle() - base_rotation;
            Some(SplitData::new(snap_data.position,area,cut_angle))
        } else {
            None
        }
    }

    /// Split the shape at the given `position` and highlight the given `area`. This might fail if
    /// the given position is too far from the shape. In that case, splitting is disabled.
    fn enable_hover_split(&self, position:Vector2<f32>, area:Area) {
        if let Some(split_data) = self.get_split_data_for_hover(position,area) {
            self.edge_shape_views().iter().for_each(|shape| shape.enable_hover_split(split_data));
        } else {
            self.disable_hover_split()
        }
    }

    fn disable_hover_split(&self) {
        for shape in self.edge_shape_views() {
            shape.disable_hover_split()
        }
    }
}



// =========================
// === Shape Definitions ===
// =========================

/// SplitShape allows a shape to be split along a line and each sub-shape to be colored separately.
struct SplitShape {
    part_a : AnyShape,
    part_b : AnyShape,
    joint  : AnyShape,
}

impl SplitShape {
    /// Splits the shape in two at the line given by the offset and rotation. Will render a
    /// circular "joint" at the given `offset`, if `joint_radius` > 0.0.
    fn new
    (base_shape:AnyShape, offset:&Var<Vector2<f32>>,rotation:&Var<f32>,joint_radius:&Var<Distance<Pixels>>) -> Self {
        let offset_x    = Var::<Distance<Pixels>>::from(offset.x());
        let offset_y    = Var::<Distance<Pixels>>::from(offset.y());
        let rotation    = Var::<Angle<Radians>>::from(rotation.clone());
        let split_plane = HalfPlane()
            .rotate(&rotation)
            .translate_x(&offset_x)
            .translate_y(&offset_y);

        let part_a = base_shape.intersection(&split_plane).into();
        let part_b = base_shape.difference(&split_plane).into();

        let joint_radius = Var::<Distance<Pixels>>::from(joint_radius);
        let joint        = Circle(joint_radius)
                            .translate_x(&offset_x)
                            .translate_y(&offset_y)
                            .into();

        SplitShape { part_a,part_b,joint }
    }

    /// Fill the two parts and the joint and return the combined shape.
    fn fill<Color:Into<color::Rgba>>(&self, color_a:Color, color_b:Color) -> AnyShape {
        let color_a = color_a.into();
        let color_b = color_b.into();

        let part_a_filled = self.part_a.fill(&color_a);
        let part_b_filled = self.part_b.fill(&color_b);
        let joint_filled  = self.joint.fill(&color_a);
        (part_a_filled + part_b_filled + joint_filled).into()
    }
}

macro_rules! define_corner_start {($color:expr, $highlight_color:expr) => {
    /// Shape definition.
    pub mod corner {
        use super::*;
        ensogl::define_shape_system! {
            (radius:f32, angle:f32, start_angle:f32, pos:Vector2<f32>, dim:Vector2<f32>,
             hover_split_offset:Vector2<f32>,hover_split_rotation:f32) {
                let radius = 1.px() * radius;
                let width  = LINE_WIDTH.px();
                let width2 = &width / 2.0;
                let ring   = Circle(&radius + &width2) - Circle(radius-width2);
                let right : Var<f32> = (RIGHT_ANGLE).into();
                let rot    = right - &angle/2.0 + start_angle;
                let mask   = Plane().cut_angle_fast(angle).rotate(rot);
                let shape  = ring * mask;

                let shadow_size = 10.px();
                let n_radius = &shadow_size + 1.px() * dim.y();
                let n_shape  = Rect(
                    (&shadow_size*2.0 + 2.px() * dim.x(),&n_radius*2.0)).corners_radius(n_radius);
                let n_shape  = n_shape.fill(color::Rgba::new(1.0,0.0,0.0,1.0));
                let tx       = - 1.px() * pos.x();
                let ty       = - 1.px() * pos.y();
                let n_shape  = n_shape.translate((tx,ty));

                let shape    = shape - n_shape;

                let split_shape = SplitShape::new(
                    shape.into(),&(&hover_split_offset).into(),&hover_split_rotation.into(),&(width * 0.5));
                let shape       = split_shape.fill($color, $highlight_color);
                shape.into()
            }
        }

        impl EdgeShape for component::ShapeView<Shape> {
            fn set_hover_split_offset(&self, offset:Vector2<f32>) {
               self.shape.hover_split_offset.set(offset);
            }

            fn set_hover_split_rotation(&self, angle:f32) {
                self.shape.hover_split_rotation.set(angle);
            }

            fn sprite(&self) -> &Sprite{
                &self.shape.sprite
            }

            fn snap_to_shape_local(&self, _point:Vector2<f32>) -> Option<Vector2<f32>> {
                // FIXME this needs to be implemented for upwards edges
                None
            }

        }
    }
}}

macro_rules! define_corner_end {($color:expr, $highlight_color:expr) => {
    /// Shape definition.
    pub mod corner {
        use super::*;
        ensogl::define_shape_system! {
            (radius:f32, angle:f32, start_angle:f32, pos:Vector2<f32>, dim:Vector2<f32>,
             hover_split_offset:Vector2<f32>,hover_split_rotation:f32) {
                let radius = 1.px() * radius;
                let width  = LINE_WIDTH.px();
                let width2 = &width / 2.0;
                let ring   = Circle(&radius + &width2) - Circle(radius-width2);
                let right : Var<f32> = (RIGHT_ANGLE).into();
                let rot    = right - &angle/2.0 + start_angle;
                let mask   = Plane().cut_angle_fast(angle).rotate(rot);
                let shape  = ring * mask;

                let shadow_size = 10.px() + 1.px();
                let n_radius = &shadow_size + 1.px() * dim.y();
                let n_shape  = Rect(
                (&shadow_size*2.0 + 2.px() * dim.x(),&n_radius*2.0)).corners_radius(n_radius);
                let n_shape  = n_shape.fill(color::Rgba::new(1.0,0.0,0.0,1.0));
                let tx       = - 1.px() * pos.x();
                let ty       = - 1.px() * pos.y();
                let n_shape  = n_shape.translate((tx,ty));

                let shape = shape * n_shape;
                let split_shape = SplitShape::new(
                shape.into(),&hover_split_offset.into(),&hover_split_rotation.into(),&(width * 0.5));
                let shape       = split_shape.fill($color, $highlight_color);

                shape.into()
            }
        }

        impl EdgeShape for component::ShapeView<Shape> {
            fn set_hover_split_offset(&self, offset:Vector2<f32>) {
                self.shape.hover_split_offset.set(offset);
            }

            fn set_hover_split_rotation(&self, angle:f32) {
                 self.shape.hover_split_rotation.set(angle);
            }

            fn sprite(&self) -> &Sprite{
                &self.shape.sprite
            }

            fn cut_angle_for_point_local(&self, point:Vector2<f32>) -> nalgebra::Rotation2<f32> {
                nalgebra::Rotation2::rotation_between(&point,&Vector2::new(1.0,0.0))
            }

            fn snap_to_shape_local(&self, point:Vector2<f32>) -> Option<Vector2<f32>> {
                // This shape is only valid for the upper part of the circle.
                if point.y < 0.0 {
                    return None
                }
                let radius = self.shape.radius.get();
                let center = Vector2::zero();
                let point_to_center = point.xy() - center;

                // Note: this is the closes point to the full circle. Will fail to produce the
                // desired result if the given point is closer to another part of the circle, than
                // this curve. We need to validate in some other context, that we are in the correct
                // quadrant.
                let closest_point = center + point_to_center / point_to_center.magnitude() * radius;

                Some(Vector2::new(closest_point.x, closest_point.y))
           }
        }
    }
}}

macro_rules! define_line {($color:expr, $highlight_color:expr) => {
    /// Shape definition.
    pub mod line {
        use super::*;
        ensogl::define_shape_system! {
            (hover_split_offset:Vector2<f32>,hover_split_rotation:f32,split_joint_position:Vector2<f32>) {
                let width  = LINE_WIDTH.px();
                let height : Var<Pixels> = "input_size.y".into();
                let shape  = Rect((width.clone(),height));

                let split_shape = SplitShape::new(
                    shape.into(),&hover_split_offset.into(),&hover_split_rotation.into(),&(&width * 0.5));
                let shape       = split_shape.fill($color, $highlight_color);
                shape.into()
            }
        }

        impl EdgeShape for component::ShapeView<Shape> {
            fn set_hover_split_offset(&self, offset:Vector2<f32>) {
                self.shape.hover_split_offset.set(offset);
            }

            fn set_hover_split_rotation(&self, angle:f32) {
                self.shape.hover_split_rotation.set(angle);
            }

            fn sprite(&self) -> &Sprite{
                &self.shape.sprite
            }

            fn snap_to_shape_local(&self, point:Vector2<f32>) -> Option<Vector2<f32>> {
               Some(Vector2::new(0.0, point.y))
            }
        }
    }
}}

macro_rules! define_arrow {($color:expr, $highlight_color:expr) => {
    /// Shape definition.
    pub mod arrow {
        use super::*;
        ensogl::define_shape_system! {
            (hover_split_offset:Vector2<f32>,hover_split_rotation:f32,split_joint_position:Vector2<f32>) {
                let width  : Var<Pixels> = "input_size.x".into();
                let height : Var<Pixels> = "input_size.y".into();
                let width      = width  - (2.0 * PADDING).px();
                let height     = height - (2.0 * PADDING).px();
                let triangle   = Triangle(width,height);
                let offset     = (LINE_WIDTH/2.0).px();
                let triangle_l = triangle.translate_x(-&offset);
                let triangle_r = triangle.translate_x(&offset);
                let shape      = triangle_l + triangle_r;

                let split_shape  = SplitShape::new(
                  shape.into(),&hover_split_offset.into(),&hover_split_rotation.into(),&0.0.px());
                let shape       = split_shape.fill($color, $highlight_color);
                shape.into()
            }
        }

        impl EdgeShape for component::ShapeView<Shape> {
            fn set_hover_split_offset(&self, offset:Vector2<f32>) {
                self.shape.hover_split_offset.set(offset);
            }

            fn set_hover_split_rotation(&self, angle:f32) {
                 self.shape.hover_split_rotation.set(angle);
            }

            fn sprite(&self) -> &Sprite{
                &self.shape.sprite
            }

            fn snap_to_shape_local(&self, _point:Vector2<f32>) -> Option<Vector2<f32>> {
               None
            }
        }
    }
}}



// ========================
// === Shape Operations ===
// ========================

trait LayoutLine {
    fn layout_v(&self,start:Vector2<f32>,len:f32);
    fn layout_h(&self,start:Vector2<f32>,len:f32);
    fn layout_v_no_overlap(&self,start:Vector2<f32>,len:f32);
    fn layout_h_no_overlap(&self,start:Vector2<f32>,len:f32);
}

impl LayoutLine for component::ShapeView<front::line::Shape> {
    fn layout_v(&self, start:Vector2<f32>, len:f32) {
        let pos  = Vector2::new(start.x, start.y + len/2.0);
        let size = Vector2::new(LINE_SHAPE_WIDTH, len.abs()+LINE_SIDES_OVERLAP);
        self.shape.sprite.size.set(size);
        self.set_position_xy(pos);
    }
    fn layout_h(&self, start:Vector2<f32>, len:f32) {
        let pos  = Vector2::new(start.x + len/2.0, start.y);
        let size = Vector2::new(LINE_SHAPE_WIDTH, len.abs()+LINE_SIDES_OVERLAP);
        self.shape.sprite.size.set(size);
        self.set_position_xy(pos);
    }
    fn layout_v_no_overlap(&self, start:Vector2<f32>, len:f32) {
        let pos  = Vector2::new(start.x, start.y + len/2.0);
        let size = Vector2::new(LINE_SHAPE_WIDTH, len.abs());
        self.shape.sprite.size.set(size);
        self.set_position_xy(pos);
    }
    fn layout_h_no_overlap(&self, start:Vector2<f32>, len:f32) {
        let pos  = Vector2::new(start.x + len/2.0, start.y);
        let size = Vector2::new(LINE_SHAPE_WIDTH, len.abs());
        self.shape.sprite.size.set(size);
        self.set_position_xy(pos);
    }
}

impl LayoutLine for component::ShapeView<back::line::Shape> {
    fn layout_v(&self, start:Vector2<f32>, len:f32) {
        let pos  = Vector2::new(start.x, start.y + len/2.0);
        let size = Vector2::new(LINE_SHAPE_WIDTH, len.abs()+LINE_SIDES_OVERLAP);
        self.shape.sprite.size.set(size);
        self.set_position_xy(pos);
    }
    fn layout_h(&self, start:Vector2<f32>, len:f32) {
        let pos  = Vector2::new(start.x + len/2.0, start.y);
        let size = Vector2::new(LINE_SHAPE_WIDTH, len.abs()+LINE_SIDES_OVERLAP);
        self.shape.sprite.size.set(size);
        self.set_position_xy(pos);
    }
    fn layout_v_no_overlap(&self, start:Vector2<f32>, len:f32) {
        let pos  = Vector2::new(start.x, start.y + len/2.0);
        let size = Vector2::new(LINE_SHAPE_WIDTH, len.abs());
        self.shape.sprite.size.set(size);
        self.set_position_xy(pos);
    }
    fn layout_h_no_overlap(&self, start:Vector2<f32>, len:f32) {
        let pos  = Vector2::new(start.x + len/2.0, start.y);
        let size = Vector2::new(LINE_SHAPE_WIDTH, len.abs());
        self.shape.sprite.size.set(size);
        self.set_position_xy(pos);
    }
}



// ===========================
// === Front / Back Shapes ===
// ===========================

/// Shape definitions which will be rendered in the front layer (on top of nodes).
pub mod front {
    use super::*;
    define_corner_start!(color::Lcha::new(0.6,0.5,0.76,1.0),color::Lcha::new(0.2,0.5,0.76,1.0));
    define_line!(color::Lcha::new(0.6,0.5,0.76,1.0),color::Lcha::new(0.2,0.5,0.76,1.0));
    define_arrow!(color::Lcha::new(0.6,0.5,0.76,1.0),color::Lcha::new(0.2,0.5,0.76,1.0));
}

/// Shape definitions which will be rendered in the bottom layer (below nodes).
pub mod back {
    use super::*;
    define_corner_end!(color::Lcha::new(0.6,0.5,0.76,1.0),color::Lcha::new(0.2,0.5,0.76,1.0));
    define_line!(color::Lcha::new(0.6,0.5,0.76,1.0),color::Lcha::new(0.2,0.5,0.76,1.0));
    define_arrow!(color::Lcha::new(0.6,0.5,0.76,1.0),color::Lcha::new(0.2,0.5,0.76,1.0));
}



// ===========================
// === Front / Back Layers ===
// ===========================

macro_rules! define_components {
    ($name:ident {
        $($field:ident : $field_type:ty),* $(,)?
    }) => {
        #[derive(Debug,Clone,CloneRef)]
        #[allow(missing_docs)]
        pub struct $name {
            pub logger            : Logger,
            pub display_object    : display::object::Instance,
            pub shape_view_events : Rc<Vec<ShapeViewEvents>>,
            $(pub $field : component::ShapeView<$field_type>),*
        }

        impl $name {
            /// Constructor.
            pub fn new(logger:Logger, scene:&Scene) -> Self {
                let display_object = display::object::Instance::new(&logger);
                $(let $field = component::ShapeView::new(
                    Logger::sub(&logger,stringify!($field)),scene);)*
                    $(display_object.add_child(&$field);)*
                let mut shape_view_events:Vec<ShapeViewEvents> = Vec::default();
                $(shape_view_events.push($field.events.clone_ref());)*
                let shape_view_events = Rc::new(shape_view_events);

                Self {logger,display_object,shape_view_events,$($field),*}
            }


        }

        impl display::Object for $name {
            fn display_object(&self) -> &display::object::Instance {
                &self.display_object
            }
        }

        impl MultiShape for $name {
            fn events(&self) -> Vec<ShapeViewEvents> {
                self.shape_view_events.to_vec()
            }

            fn edge_shape_views(&self) -> Vec<&dyn EdgeShape> {
                let mut output = Vec::<&dyn EdgeShape>::default();
                $(output.push(&self.$field);)*
                output
            }
        }
    }
}

define_components!{
    Front {
        corner     : front::corner::Shape,
        corner2    : front::corner::Shape,
        corner3    : front::corner::Shape,
        side_line  : front::line::Shape,
        side_line2 : front::line::Shape,
        main_line  : front::line::Shape,
        port_line  : front::line::Shape,
        arrow      : front::arrow::Shape,
    }
}

define_components!{
    Back {
        corner     : back::corner::Shape,
        corner2    : back::corner::Shape,
        corner3    : back::corner::Shape,
        side_line  : back::line::Shape,
        side_line2 : back::line::Shape,
        main_line  : back::line::Shape,
        arrow      : back::arrow::Shape,
    }
}

// TODO consider a nice design here that is more general
impl MultiShape for (&Front, &Back) {
    fn events(&self) -> Vec<ShapeViewEvents> {
        let mut events_back:Vec<ShapeViewEvents>  = self.0.events();
        let mut events_front:Vec<ShapeViewEvents> = self.1.events();
        events_front.append(&mut events_back);
        events_front
    }

    fn edge_shape_views(&self) -> Vec<&dyn EdgeShape> {
        let mut shapes_back  = self.0.edge_shape_views();
        let mut shapes_front = self.1.edge_shape_views();
        shapes_front.append(&mut shapes_back);
        shapes_front
    }
}

// TODO: Implement proper sorting and remove.
/// Hack function used to register the elements for the sorting purposes. To be removed.
pub fn sort_hack_1(scene:&Scene) {
    let logger = Logger::new("hack_sort");
    component::ShapeView::<back::corner::Shape>::new(&logger,scene);
    component::ShapeView::<back::line::Shape>::new(&logger,scene);
    component::ShapeView::<back::arrow::Shape>::new(&logger,scene);
}

// TODO: Implement proper sorting and remove.
/// Hack function used to register the elements for the sorting purposes. To be removed.
pub fn sort_hack_2(scene:&Scene) {
    let logger = Logger::new("hack_sort");
    component::ShapeView::<front::corner::Shape>::new(&logger,scene);
    component::ShapeView::<front::line::Shape>::new(&logger,scene);
    component::ShapeView::<front::arrow::Shape>::new(&logger,scene);
}



// ===========
// === FRP ===
// ===========

/// FRP endpoints for aggregated mouse events.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct ShapeViewEventsProxy {
    pub mouse_down : frp::Stream<()>,
    pub mouse_over : frp::Stream<()>,
    pub mouse_out  : frp::Stream<()>,

    on_mouse_down : frp::Source<()>,
    on_mouse_over : frp::Source<()>,
    on_mouse_out  : frp::Source<()>,
}

#[allow(missing_docs)]
impl ShapeViewEventsProxy {
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! { network
            def on_mouse_over  = source();
            def on_mouse_out   = source();
            def on_mouse_down  = source();
        }
        let mouse_down = on_mouse_down.clone_ref().into();
        let mouse_over = on_mouse_over.clone_ref().into();
        let mouse_out  = on_mouse_out.clone_ref().into();
        Self {on_mouse_down,mouse_down,mouse_over,mouse_out,on_mouse_out,on_mouse_over}
    }
}


/// FRP endpoints of the edge.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Frp {
    pub source_width    : frp::Source<f32>,
    pub source_height   : frp::Source<f32>,
    pub target_position : frp::Source<Vector2>,
    pub target_attached : frp::Source<bool>,
    pub redraw          : frp::Source<()>,

    pub hover_position  : frp::Source<Option<Vector2<f32>>>,
    pub shape_events    : ShapeViewEventsProxy
}

impl Frp {
    /// Constructor.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! { network
            def source_width    = source();
            def source_height   = source();
            def target_position = source();
            def target_attached = source();
            def redraw          = source();
            def hover_position  = source();
        }
        let shape_events = ShapeViewEventsProxy::new(&network);
        Self {source_width,source_height,target_position,target_attached,redraw,
              hover_position,shape_events}
    }
}



// ==================
// === Math Utils ===
// ==================

/// For the given radius of the first circle (`r1`), radius of the second circle (`r2`), and the
/// x-axis position of the second circle (`x`), computes the y-axis position of the second circle in
/// such a way, that the borders of the circle cross at the right angle. It also computes the angle
/// of the intersection. Please note, that the center of the first circle is in the origin.
///
/// ```compile_fail
///       r1
///      ◄───►                (1) x^2 + y^2 = r1^2 + r2^2
///    _____                  (1) => y = sqrt((r1^2 + r2^2)/x^2)
///  .'     `.
/// /   _.-"""B-._     ▲
/// | .'0┼    |   `.   │      angle1 = A-XY-0
/// \/   │    /     \  │ r2   angle2 = 0-XY-B
/// |`._ │__.'       | │      alpha  = B-XY-X_AXIS
/// |   A└───┼─      | ▼
/// |      (x,y)     |        tg(angle1) = y  / x
///  \              /         tg(angle2) = r1 / r2
///   `._        _.'          alpha      = PI - angle1 - angle2
///      `-....-'
///```
fn circle_intersection(x:f32, r1:f32, r2:f32) -> (f32,f32) {
    let x_norm = x.clamp(-r2,r1);
    let y      = (r1*r1 + r2*r2 - x_norm*x_norm).sqrt();
    let angle1 = f32::atan2(y,x_norm);
    let angle2 = f32::atan2(r1,r2);
    let angle  = std::f32::consts::PI - angle1 - angle2;
    (y,angle)
}



// ============
// === Edge ===
// ============

/// Edge definition.
#[derive(AsRef,Clone,CloneRef,Debug,Deref)]
pub struct Edge {
    #[deref]
    model   : Rc<EdgeModel>,
    network : frp::Network,
}

impl AsRef<Edge> for Edge {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl display::Object for EdgeModelData {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}

impl Edge {
    /// Constructor.
    pub fn new(scene:&Scene) -> Self {
        let network = frp::Network::new();
        let data    = Rc::new(EdgeModelData::new(scene,&network));
        let model   = Rc::new(EdgeModel {data});
        Self {model,network} . init()
    }

    fn init(self) -> Self {
        let network         = &self.network;
        let input           = &self.frp;
        let target_position = &self.target_position;
        let target_attached = &self.target_attached;
        let source_width    = &self.source_width;
        let source_height   = &self.source_height;
        let hover_position  = &self.hover_position;
        let model           = &self.model;

        model.data.front.register_proxy_frp(network, &input.shape_events);
        model.data.back.register_proxy_frp(network, &input.shape_events);

        frp::extend! { network
            eval input.target_position ((t) target_position.set(*t));
            eval input.target_attached ((t) target_attached.set(*t));
            eval input.source_width    ((t) source_width.set(*t));
            eval input.source_height   ((t) source_height.set(*t));
            eval input.hover_position  ((t) hover_position.set(*t));
            eval_ input.redraw (model.redraw());
        }

        self
    }
}

impl display::Object for Edge {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}



// =================
// === EdgeModel ===
// =================

/// Indicates the type of end connection of the Edge.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
#[allow(missing_docs)]
pub enum EndDesignation {
    Input,
    Output
}

/// Edge definition.
#[derive(AsRef,Clone,CloneRef,Debug,Deref)]
pub struct EdgeModel {
    data : Rc<EdgeModelData>,
}

/// Internal data of `Edge`
#[derive(Debug)]
#[allow(missing_docs)]
pub struct EdgeModelData {
    pub display_object  : display::object::Instance,
    pub logger          : Logger,
    pub frp             : Frp,
    pub front           : Front,
    pub back            : Back,
    pub source_width    : Rc<Cell<f32>>,
    pub source_height   : Rc<Cell<f32>>,
    pub target_position : Rc<Cell<Vector2>>,
    pub target_attached : Rc<Cell<bool>>,
    hover_position      : Rc<Cell<Option<Vector2<f32>>>>,
}

impl EdgeModelData {
    /// Constructor.
    pub fn new(scene:&Scene, network:&frp::Network) -> Self {
        let logger         = Logger::new("edge");
        let display_object = display::object::Instance::new(&logger);
        let front          = Front::new(Logger::sub(&logger,"front"),scene);
        let back           = Back::new (Logger::sub(&logger,"back"),scene);

        display_object.add_child(&front);
        display_object.add_child(&back);

        front . side_line  . mod_rotation(|r| r.z = RIGHT_ANGLE);
        back  . side_line  . mod_rotation(|r| r.z = RIGHT_ANGLE);
        front . side_line2 . mod_rotation(|r| r.z = RIGHT_ANGLE);
        back  . side_line2 . mod_rotation(|r| r.z = RIGHT_ANGLE);

        let frp             = Frp::new(&network);
        let source_height   = default();
        let source_width    = default();
        let target_position = default();
        let target_attached = Rc::new(Cell::new(false));
        let hover_position  = default();

        Self {display_object,logger,frp,front,back,source_width,source_height,target_position,
              target_attached,hover_position}
    }

    fn is_in_upper_half(&self, point:Vector2<f32>) -> bool {
        let start = nalgebra::Point2::from(self.display_object.position().xy());
        let end   = nalgebra::Point2::from(self.target_position.get());
        let point = nalgebra::Point2::from(point);
        let mid_y = (start.y + end.y) / 2.0;
        point.y > mid_y
    }

    /// Returns whether the given hover position belongs to the Input or Output part of the
    /// edge.
    pub fn end_designation_for_position(&self, point:Vector2<f32>) -> EndDesignation {
        let target_y        = self.display_object.position().y;
        let source_y        = self.target_position.get().y;
        let delta_y         = target_y - source_y;
        let input_above_mid = delta_y > 0.0;
        let point_area      = self.hover_split_area_for_position(point);

        match (point_area, input_above_mid) {
            (Area::Above, true)  => EndDesignation::Input,
            (Area::Above, false) => EndDesignation::Output,
            (Area::Below, true)  => EndDesignation::Output,
            (Area::Below, false) => EndDesignation::Input,
        }
    }

    /// Returns whether the given positions should change the color of the area above or below.
    fn hover_split_area_for_position(&self, point:Vector2<f32>) -> Area {
        if self.is_in_upper_half(point) {
            Area::Above
        } else {
            Area::Below
        }
    }

    /// Shows whether the correct hover split is implemented for the given configuration
    /// This will no longer needed once it is implemented for all quadrants.
    fn can_show_hover_split(&self) -> bool {
        // FIXME remove this once the hover edge splitting is completely implemented.
        let target_pos = self.target_position.get();
        let source_pos = self.display_object.position();
        let is_below = target_pos.y <source_pos.y;
        let is_left  = target_pos.x < source_pos.x;
        match (is_below, is_left) {
            (true, true)   => true,
            (true, false)  => true,
            (false, true)  => false,
            (false, false) => false,
        }
    }

    /// Redraws the connection.
    #[allow(clippy::cognitive_complexity)]
    pub fn redraw(&self) {

        // === Variables ===

        let fg               = &self.front;
        let bg               = &self.back;
        let target_attached  = self.target_attached.get();
        let node_half_width  = self.source_width.get() / 2.0;
        let node_half_height = self.source_height.get() / 2.0;
        let node_circle      = Vector2::new(node_half_width-node_half_height,0.0);
        let node_radius      = node_half_height;

        // === Update Highlights ===

        if self.can_show_hover_split() {
            let hover_position  = self.hover_position.get();
            if let Some(hover_position) = hover_position {
                let highlight_area = self.hover_split_area_for_position(hover_position);
                // This call treats both `Back` and `Front` as a combined `MultiShape`.
                (fg,bg).enable_hover_split(hover_position,highlight_area);
            } else {
                (fg,bg).disable_hover_split();
            }
        } else {
            (fg,bg).disable_hover_split();
        }


        // === Target ===
        //
        // Target is the end position of the connection in local node space (the origin is placed in
        // the center of the node). We handle lines drawing in special way when target is below the
        // node (for example, not to draw the port line above source node).
        //
        // ╭──────────────╮
        // │      ┼ (0,0) │
        // ╰──────────────╯────╮
        //                     │
        //                     ▢ target

        let world_space_target     = self.target_position.get();
        let target_x               = world_space_target.x - self.position().x;
        let target_y               = world_space_target.y - self.position().y;
        let side                   = target_x.signum();
        let target_x               = target_x.abs();
        let target                 = Vector2::new(target_x,target_y);
        let target_is_below_node_x = target.x < node_half_width;
        let target_is_below_node_y = target.y < (-node_half_height);
        let target_is_below_node   = target_is_below_node_x && target_is_below_node_y;
        let port_line_len_max      = node_half_height + NODE_PADDING;
        let side_right_angle       = side * RIGHT_ANGLE;


        // === Upward Discovery ===
        //
        // Discovers when the connection should go upwards. The `upward_corner_radius` defines the
        // preferred radius for every corner in this scenario.
        //
        // ╭─────╮    ╭─╮
        // ╰─────╯────╯ │
        //              ▢

        let upward_corner_radius        = 20.0;
        let min_len_for_non_curved_line = upward_corner_radius + port_line_len_max;
        let upward_distance             = target.y + min_len_for_non_curved_line;


        // === Flat side ===
        //
        // Maximum side distance before connection is curved up.
        //
        // ╭─────╮◄──►    ╭─────╮◄──►╭─╮
        // ╰─────╯───╮    ╰─────╯────╯ │
        //           ▢                 ▢

        let flat_side_size = 40.0;
        let is_flat_side   = target.x < node_half_width + flat_side_size;
        let downward_flat  = if target_is_below_node_x {target_is_below_node_y} else {target.y<0.0};
        let downward_far   = -target.y > min_len_for_non_curved_line || target_is_below_node;
        let is_down        = if is_flat_side {downward_flat} else {downward_far};


        // === Port Line Length ===
        //
        // ╭──╮
        // ╰──╯───╮
        //        ╵
        //     ╭──┼──╮ ▲  Port line covers the area above target node and the area of target node
        //     │  ▢  │ ▼  shadow. It can be shorter if the target position is below the node or the
        //     ╰─────╯    connection is being dragged, in order not to overlap with the source node.

        let port_line_start    = Vector2::new(side * target.x, target.y + MOUSE_OFFSET);
        let space_attached     = -port_line_start.y - node_half_height - LINE_SIDE_OVERLAP;
        let space              = space_attached - NODE_PADDING;
        let len_below_free     = max(0.0,min(port_line_len_max,space));
        let len_below_attached = max(0.0,min(port_line_len_max,space_attached));
        let len_below          = if target_attached {len_below_attached} else {len_below_free};
        let far_side_len       = if target_is_below_node {len_below} else {port_line_len_max};
        let flat_side_len      = min(far_side_len,-port_line_start.y);
        let mut port_line_len  = if is_flat_side && is_down {flat_side_len} else {far_side_len};
        let port_line_end      = Vector2::new(target.x,port_line_start.y + port_line_len);


        // === Corner1 ===
        //
        // The first corner on the line. It is always placed at the right angle to the tangent of
        // the node border. In case the edge is in the drag mode, the curve is divided into two
        // parts. The first part is placed under the source node shadow, while the second part is
        // placed on the top layer.
        //
        // ╭─────╮        ╭─────╮ 2╭──╮3
        // ╰─────╯──╮1    ╰─────╯──╯1 ▢
        //          ▢

        let mut corner1_target = port_line_end;
        if !is_down {
            corner1_target.x = if is_flat_side {
                let radius_grow = max(0.0,target.x - node_half_width + upward_corner_radius);
                node_half_width + upward_corner_radius + radius_grow
            } else {
                let radius1 = node_half_width + (target.x - node_half_width)/2.0;
                let radius2 = node_half_width + 2.0*upward_corner_radius;
                min(radius1,radius2)
            };
            corner1_target.y = min(upward_corner_radius,upward_distance/2.0);
        }

        let corner1_grow   = ((corner1_target.x - node_half_width) * 0.6).max(0.0);
        let corner1_radius = 20.0 + corner1_grow;
        let corner1_radius = corner1_radius.min(corner1_target.y.abs());
        let corner1_x      = corner1_target.x - corner1_radius;

        let corner1_x_loc       = corner1_x - node_circle.x;
        let (y,angle)           = circle_intersection(corner1_x_loc,node_radius,corner1_radius);
        let corner1_y           = if is_down {-y} else {y};
        let corner1             = Vector2::new(corner1_x*side, corner1_y);
        let angle_overlap       = if corner1_x > node_half_width { 0.0 } else { 0.1 };
        let corner1_side        = (corner1_radius + PADDING) * 2.0;
        let corner1_size        = Vector2::new(corner1_side,corner1_side);
        let corner1_start_angle = if is_down {0.0} else {side_right_angle};
        let corner1_angle       = (angle + angle_overlap) * side;
        let corner1_angle       = if is_down {corner1_angle} else {side_right_angle};

        bg.corner.shape.sprite.size.set(corner1_size);
        bg.corner.shape.start_angle.set(corner1_start_angle);
        bg.corner.shape.angle.set(corner1_angle);
        bg.corner.shape.radius.set(corner1_radius);
        bg.corner.shape.pos.set(corner1);
        bg.corner.set_position_xy(corner1);
        if !target_attached {
            bg.corner.shape.dim.set(Vector2::new(node_half_width,node_half_height));
            fg.corner.shape.sprite.size.set(corner1_size);
            fg.corner.shape.start_angle.set(corner1_start_angle);
            fg.corner.shape.angle.set(corner1_angle);
            fg.corner.shape.radius.set(corner1_radius);
            fg.corner.shape.pos.set(corner1);
            fg.corner.shape.dim.set(Vector2::new(node_half_width,node_half_height));
            fg.corner.set_position_xy(corner1);
        } else {
            fg.corner.shape.sprite.size.set(zero());
            bg.corner.shape.dim.set(Vector2::new(INFINITE,INFINITE));
        }


        // === Side Line ===
        //
        // Side line is the first horizontal line. In case the edge is in drag mode, the line is
        // divided into two segments. The first is placed below the shadow of the source node, while
        // the second is placed on the top layer. The side line placement is the same in case of
        // upwards connections - it is then placed between node and corenr 1.
        //
        // ╭─────╮       ╭─────╮ 2╭──╮3
        // ╰─────╯╴──╮   ╰─────╯╴─╯1 ▢
        //           ▢

        let side_line_shift = LINE_SIDES_OVERLAP;
        let side_line_len   = max(0.0,corner1_x - node_half_width + side_line_shift);
        let bg_line_x       = node_half_width - side_line_shift;
        let bg_line_start   = Vector2::new(side*bg_line_x,0.0);
        if target_attached {
            let bg_line_len = side*side_line_len;
            fg.side_line.shape.sprite.size.set(zero());
            bg.side_line.layout_h(bg_line_start,bg_line_len);
        } else {
            let bg_max_len            = NODE_PADDING + side_line_shift;
            let bg_line_len           = min(side_line_len,bg_max_len);
            let bg_end_x              = bg_line_x + bg_line_len;
            let fg_line_start         = Vector2::new(side*(bg_end_x+LINE_SIDE_OVERLAP),0.0);
            let fg_line_len           = side*(side_line_len - bg_line_len);
            let bg_line_len_overlap   = side * min(side_line_len,bg_max_len+LINE_SIDES_OVERLAP);
            bg.side_line.layout_h(bg_line_start,bg_line_len_overlap);
            fg.side_line.layout_h_no_overlap(fg_line_start,fg_line_len);
        }


        // === Main Line (downwards) ===
        //
        // Main line is the long vertical line. In case it is placed below the node and the edge is
        // in drag mode, it is divided into two segments. The upper segment is drawn behind node
        // shadow, while the second is drawn on the top layer. In case of edge in drag mode drawn
        // next to node, only the top layer segment is used.
        //
        // Please note that only applies to edges going down. Refer to docs of main line of edges
        // going up to learn more.
        //
        // Double edge:   Single edge:
        // ╭─────╮        ╭─────╮
        // ╰──┬──╯        ╰─────╯────╮
        //    ╷                      │
        //    ▢                      ▢

        if is_down {
            let main_line_end_y = corner1.y;
            let main_line_len   = main_line_end_y - port_line_start.y;
            if !target_attached && target_is_below_node {
                let back_line_start_y = max(-node_half_height - NODE_PADDING, port_line_start.y);
                let back_line_start = Vector2::new(port_line_start.x, back_line_start_y);
                let back_line_len = main_line_end_y - back_line_start_y;
                let front_line_len = main_line_len - back_line_len;
                bg.main_line.layout_v(back_line_start, back_line_len);
                fg.main_line.layout_v(port_line_start, front_line_len);
            } else if target_attached {
                let main_line_start_y = port_line_start.y + port_line_len;
                let main_line_start = Vector2::new(port_line_start.x, main_line_start_y);
                fg.main_line.shape.sprite.size.set(zero());
                bg.main_line.layout_v(main_line_start, main_line_len - port_line_len);
            } else {
                bg.main_line.shape.sprite.size.set(zero());
                fg.main_line.layout_v(port_line_start, main_line_len);
            }
        }


        if !is_down {

            // === Corner2 & Corner3 Radius ===
            //
            // ╭─────╮ 2╭──╮3
            // ╰─────╯──╯1 ▢

            let corner2_radius      = corner1_radius;
            let corner3_radius      = upward_corner_radius;

            let corner2_x           = corner1_target.x + corner1_radius;
            let corner3_x           = port_line_end.x - corner3_radius;
            let corner2_bbox_x      = corner2_x - corner2_radius;
            let corner3_bbox_x      = corner3_x + corner3_radius;

            let corner_2_3_dist     = corner3_bbox_x - corner2_bbox_x;
            let corner_2_3_side     = corner_2_3_dist.signum();
            let corner_2_3_dist     = corner_2_3_dist.abs();
            let corner_2_3_width    = corner2_radius + corner3_radius;
            let corner_2_3_do_scale = corner_2_3_dist < corner_2_3_width;
            let corner_2_3_scale    = corner_2_3_dist / corner_2_3_width;
            let corner_2_3_scale    = if corner_2_3_do_scale {corner_2_3_scale} else {1.0};

            let side_combined       = side * corner_2_3_side;
            let corner2_radius      = corner2_radius * corner_2_3_scale;
            let corner3_radius      = corner3_radius * corner_2_3_scale;
            let is_right_side       = (side_combined - 1.0).abs() < std::f32::EPSILON;


            // === Corner2 & Corner3 Placement ===
            //
            // ╭─────╮ 2╭──╮3
            // ╰─────╯──╯1 ▢

            let corner3_side  = (corner3_radius + PADDING) * 2.0;
            let corner3_size  = Vector2::new(corner3_side,corner3_side);
            let corner3_x     = port_line_end.x - corner_2_3_side * corner3_radius;
            let corner3_y     = port_line_end.y;
            let corner2_y     = corner3_y + corner3_radius - corner2_radius;
            let corner2_y     = max(corner2_y, corner1.y);
            let corner3_y     = max(corner3_y,corner2_y - corner3_radius + corner2_radius);
            let corner3       = Vector2::new(corner3_x*side,corner3_y);
            let corner3_angle = if is_right_side {0.0} else {-RIGHT_ANGLE};

            if target_attached {
                fg.corner3.shape.sprite.size.set(zero());
                bg.corner3.shape.sprite.size.set(corner3_size);
                bg.corner3.shape.start_angle.set(corner3_angle);
                bg.corner3.shape.angle.set(RIGHT_ANGLE);
                bg.corner3.shape.radius.set(corner3_radius);
                bg.corner3.shape.pos.set(corner3);
                bg.corner3.shape.dim.set(Vector2::new(INFINITE,INFINITE));
                bg.corner3.set_position_xy(corner3);
            } else {
                bg.corner3.shape.sprite.size.set(zero());
                fg.corner3.shape.sprite.size.set(corner3_size);
                fg.corner3.shape.start_angle.set(corner3_angle);
                fg.corner3.shape.angle.set(RIGHT_ANGLE);
                fg.corner3.shape.radius.set(corner3_radius);
                fg.corner3.shape.pos.set(corner3);
                fg.corner3.shape.dim.set(zero());
                fg.corner3.set_position_xy(corner3);
            }

            let corner2_x     = corner1_target.x + corner_2_3_side * corner2_radius;
            let corner2       = Vector2::new(corner2_x*side,corner2_y);
            let corner2_angle = if is_right_side {-RIGHT_ANGLE} else {0.0};

            if target_attached {
                fg.corner2.shape.sprite.size.set(zero());
                bg.corner2.shape.sprite.size.set(corner1_size);
                bg.corner2.shape.start_angle.set(corner2_angle);
                bg.corner2.shape.angle.set(RIGHT_ANGLE);
                bg.corner2.shape.radius.set(corner2_radius);
                bg.corner2.shape.pos.set(corner2);
                bg.corner2.shape.dim.set(Vector2::new(INFINITE,INFINITE));
                bg.corner2.set_position_xy(corner2);
            } else {
                bg.corner2.shape.sprite.size.set(zero());
                fg.corner2.shape.sprite.size.set(corner1_size);
                fg.corner2.shape.start_angle.set(corner2_angle);
                fg.corner2.shape.angle.set(RIGHT_ANGLE);
                fg.corner2.shape.radius.set(corner2_radius);
                fg.corner2.shape.pos.set(corner2);
                fg.corner2.shape.dim.set(zero());
                fg.corner2.set_position_xy(corner2);
            }


            // === Main Line (upwards) ===
            //
            // Main line is the first vertical line of the edge placed between the corner 1 and the
            // corner 2. In case the line is long enough, it has an arrow pointing up to show its
            // direction.
            //
            // ╭─────╮ 2╭──╮3
            // ╰─────╯──╯1 ▢

            let main_line_len   = corner2_y - corner1.y;
            let main_line_start = Vector2::new(side*corner1_target.x,corner1.y);

            if target_attached {
                fg.main_line.shape.sprite.size.set(zero());
                bg.main_line.layout_v(main_line_start, main_line_len);
            } else {
                bg.main_line.shape.sprite.size.set(zero());
                fg.main_line.layout_v(main_line_start, main_line_len);
            }

            if main_line_len > 2.0 {
                let arrow_y    = (corner1.y - corner1_radius + corner2_y + corner2_radius)/2.0;
                let arrow_pos  = Vector2::new(main_line_start.x, arrow_y);
                let arrow_size = Vector2::new(20.0,20.0);
                if target_attached {
                    fg.arrow.shape.sprite.size.set(zero());
                    bg.arrow.shape.sprite.size.set(arrow_size);
                    bg.arrow.set_position_xy(arrow_pos);
                } else {
                    bg.arrow.shape.sprite.size.set(zero());
                    fg.arrow.shape.sprite.size.set(arrow_size);
                    fg.arrow.set_position_xy(arrow_pos);
                }
            } else {
                bg.arrow.shape.sprite.size.set(zero());
                fg.arrow.shape.sprite.size.set(zero());
            }


            // === Side Line 2 ===
            //
            // Side line 2 is the horizontal line connecting corner 2 and corner 3.
            //
            // ╭─────╮ 2╭──╮3
            // ╰─────╯──╯1 ▢

            let side_line2_len  = side*(corner3_x - corner2_x);
            let side_line2_start  = Vector2::new(side*corner2_x,corner2_y + corner2_radius);
            if target_attached {
                fg.side_line2.shape.sprite.size.set(zero());
                bg.side_line2.layout_h(side_line2_start,side_line2_len);
            } else {
                bg.side_line2.shape.sprite.size.set(zero());
                fg.side_line2.layout_h(side_line2_start,side_line2_len);
            }

            port_line_len = corner3_y - port_line_start.y;
        } else {
            fg.arrow.shape.sprite.size.set(zero());
            bg.arrow.shape.sprite.size.set(zero());
            fg.corner3.shape.sprite.size.set(zero());
            bg.corner3.shape.sprite.size.set(zero());
            fg.corner2.shape.sprite.size.set(zero());
            bg.corner2.shape.sprite.size.set(zero());
            fg.side_line2.shape.sprite.size.set(zero());
            bg.side_line2.shape.sprite.size.set(zero());
        }


        // === Port Line ===

        fg.port_line.layout_v(port_line_start, port_line_len);
    }
}
