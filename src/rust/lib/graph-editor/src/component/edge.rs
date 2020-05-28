//! Definition of the Edge component.

#![allow(missing_docs)]
// WARNING! UNDER HEAVY DEVELOPMENT. EXPECT DRASTIC CHANGES.

use crate::prelude::*;

use enso_frp;
use enso_frp as frp;
use ensogl::data::color;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Sprite;
use ensogl::display::scene::Scene;
use ensogl::display::shape::*;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::component;

use super::node;



macro_rules! define_corner {($($color:tt)*) => {
    /// Shape definition.
    pub mod corner {
        use super::*;
        ensogl::define_shape_system! {
            (radius:f32, angle:f32, start_angle:f32, pos:Vector2<f32>, width:f32) {
                let radius = 1.px() * radius;
                let ww  = LINE_WIDTH.px();
                let width2 = ww / 2.0;
                let ring   = Circle(&radius + &width2) - Circle(radius-width2);
                let right : Var<f32> = (std::f32::consts::PI/2.0).into();
                let rot    = right - &angle/2.0 + start_angle;
                let mask   = Plane().cut_angle_fast(angle).rotate(rot);
                let shape  = ring * mask;

                let shadow_size = 10.px();
                let n_radius = &shadow_size + 14.px();
                let n_shape  = Rect((&shadow_size*2.0 + 2.px() * width,&n_radius*2.0)).corners_radius(n_radius);
                let n_shape  = n_shape.fill(color::Rgba::new(1.0,0.0,0.0,1.0));
                let tx       = - 1.px() * pos.x();
                let ty       = - 1.px() * pos.y();
                let n_shape  = n_shape.translate((tx,ty));

                let shape = shape - n_shape;
                let shape = shape.fill(color::Rgba::from($($color)*));

                shape.into()
            }
        }
    }
}}

macro_rules! define_corner2 {($($color:tt)*) => {
    /// Shape definition.
    pub mod corner {
        use super::*;
        ensogl::define_shape_system! {
            (radius:f32, angle:f32, start_angle:f32, pos:Vector2<f32>, dim:Vector2<f32>) {
                let radius = 1.px() * radius;
                let width  = LINE_WIDTH.px();
                let width2 = width / 2.0;
                let ring   = Circle(&radius + &width2) - Circle(radius-width2);
                let right : Var<f32> = (std::f32::consts::PI/2.0).into();
                let rot    = right - &angle/2.0 + start_angle;
                let mask   = Plane().cut_angle_fast(angle).rotate(rot);
                let shape  = ring * mask;

                let shadow_size = 10.px() + 1.px();
                let n_radius = &shadow_size + 1.px() * dim.y();
                let n_shape  = Rect((&shadow_size*2.0 + 2.px() * dim.x(),&n_radius*2.0)).corners_radius(n_radius);
                let n_shape  = n_shape.fill(color::Rgba::new(1.0,0.0,0.0,1.0));
                let tx       = - 1.px() * pos.x();
                let ty       = - 1.px() * pos.y();
                let n_shape  = n_shape.translate((tx,ty));

                let shape = shape * n_shape;
                let shape = shape.fill(color::Rgba::from($($color)*));

                shape.into()
            }
        }
    }
}}

macro_rules! define_line {($($color:tt)*) => {
    /// Shape definition.
    pub mod line {
        use super::*;
        ensogl::define_shape_system! {
            () {
                let width  = LINE_WIDTH.px();
                let height : Var<Distance<Pixels>> = "input_size.y".into();
                let shape  = Rect((width,height));
                let shape  = shape.fill(color::Rgba::from($($color)*));
                shape.into()
            }
        }
    }
}}

macro_rules! define_arrow {($($color:tt)*) => {
    /// Shape definition.
    pub mod arrow {
        use super::*;
        ensogl::define_shape_system! {
            () {
                let width  : Var<Distance<Pixels>> = "input_size.x".into();
                let height : Var<Distance<Pixels>> = "input_size.y".into();
                let width      = width  - (2.0 * PADDING).px();
                let height     = height - (2.0 * PADDING).px();
                let triangle   = Triangle(width,height);
                let offset     = (LINE_WIDTH/2.0).px();
                let triangle_l = triangle.translate_x(-&offset);
                let triangle_r = triangle.translate_x(&offset);
                let shape      = triangle_l + triangle_r;
                let shape      = shape.fill(color::Rgba::from($($color)*));
                shape.into()
            }
        }
    }
}}


// ============
// === Edge ===
// ============

pub mod front {
    use super::*;
    define_corner!(color::Lcha::new(0.6,0.5,0.76,1.0));
    define_line!(color::Lcha::new(0.6,0.5,0.76,1.0));
    define_arrow!(color::Lcha::new(0.6,0.5,0.76,1.0));
}

pub mod back {
    use super::*;
    define_corner2!(color::Lcha::new(0.6,0.5,0.76,1.0));
    define_line!(color::Lcha::new(0.6,0.5,0.76,1.0));
    define_arrow!(color::Lcha::new(0.6,0.5,0.76,1.0));
}

/// Canvas node shape definition.
pub mod helper {
    use super::*;

    ensogl::define_shape_system! {
        () {
            let shape = Circle(2.px());
            let shape = shape.fill(color::Rgba::new(1.0,0.0,0.0,1.0));
            shape.into()
        }
    }
}


const LINE_WIDTH : f32 = 4.0;
const PADDING    : f32 = 4.0;



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


#[derive(Clone,CloneRef,Debug)]
pub struct Frp {
    pub source_width    : frp::Source<f32>,
    pub target_position : frp::Source<frp::Position>,
    pub target_attached : frp::Source<bool>,
}

impl Frp {
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! { network
            def source_width    = source();
            def target_position = source();
            def target_attached = source();
        }
        Self {source_width,target_position,target_attached}
    }
}


pub fn sort_hack_1(scene:&Scene) {
    let logger = Logger::new("hack");
    component::ShapeView::<back::corner::Shape>::new(&logger,scene);
    component::ShapeView::<back::line::Shape>::new(&logger,scene);
}

pub fn sort_hack_2(scene:&Scene) {
    let logger = Logger::new("hack");
    component::ShapeView::<front::corner::Shape>::new(&logger,scene);
    component::ShapeView::<front::line::Shape>::new(&logger,scene);
}


macro_rules! define_components {
    ($name:ident {
        $($field:ident : $field_type:ty),* $(,)?
    }) => {
        #[derive(Debug,Clone,CloneRef)]
        pub struct $name {
            pub logger         : Logger,
            pub display_object : display::object::Instance,
            $(pub $field : component::ShapeView<$field_type>),*
        }

        impl $name {
            pub fn new(logger:Logger, scene:&Scene) -> Self {
                let display_object = display::object::Instance::new(&logger);
                $(let $field = component::ShapeView::new(&logger.sub(stringify!($field)),scene);)*
                $(display_object.add_child(&$field);)*
                Self {logger,display_object,$($field),*}
            }
        }

        impl display::Object for $name {
            fn display_object(&self) -> &display::object::Instance {
                &self.display_object
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
        arrow      : front::arrow::Shape,
    }
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
    pub target_position : Rc<Cell<frp::Position>>,
    pub target_attached : Rc<Cell<bool>>,
}

/// Edge definition.
#[derive(AsRef,Clone,CloneRef,Debug,Deref)]
pub struct EdgeModel {
    data : Rc<EdgeModelData>,
}

const END_OFFSET : f32 = 2.0;

const INFINITE : f32 = 99999.0;



impl display::Object for EdgeModelData {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}


impl EdgeModelData {
    pub fn redraw(&self) {
        let line_side_overlap  = 1.0;
        let line_sides_overlap = 2.0 * line_side_overlap;
        let line_shape_width   = LINE_WIDTH + 2.0 * PADDING;

        let fg              = &self.front;
        let bg              = &self.back;
        let target_attached = self.target_attached.get();

        let node_height     = node::NODE_HEIGHT;
        let node_padding    = node::SHADOW_SIZE;
        let node_radius     = node_height / 2.0;
        let node_half_width = self.source_width.get() / 2.0;
        let node_circle     = Vector2::new(node_half_width-node_radius,0.0);

        let mut port_line_len_max = node_height/2.0 + node_padding;



        let world_space_target   = self.target_position.get();
        let target_x             = world_space_target.x - self.position().x;
        let target_y             = world_space_target.y - self.position().y;
        let side                 = target_x.signum();
        let target_x             = target_x.abs();
        let target               = Vector2::new(target_x,target_y);
        let target_is_below_node = target.x < node_half_width && target.y < (-node_height/2.0);


        let mut port_line_len = if !target_is_below_node {port_line_len_max} else {
            if target_attached {
                f32::max(0.0,f32::min(port_line_len_max, -target.y - node_height/2.0))
            } else {
                f32::max(0.0,f32::min(port_line_len_max, -target.y - node_height/2.0 - node_padding))
            }
        };

        let upward_corner_radius = 20.0;


        let min_down_dist = upward_corner_radius + port_line_len;


        let upw      = target.y + min_down_dist;



        let s1 = target.x < node_half_width + 40.0;
        let downward = if s1 {
            if target.x < node_half_width {
                target.y < -node_height / 2.0
            } else {
                target.y < 0.0
            }
        } else {
            upw < 0.0 || target_is_below_node
        };

        if s1 && downward {
            port_line_len = f32::min(port_line_len,-target.y);
            port_line_len_max = f32::min(port_line_len_max,-target.y);
        }

        let mut port_line_target_y         = target.y + port_line_len_max;




        let port_line_len_diff = port_line_len_max - port_line_len;


        let mut port_line_target       = Vector2::new(target.x,port_line_target_y);
        let main_line_target = Vector2::new(target.x,port_line_target_y+port_line_len_diff);



//                let downward = -target.y > port_line_len; //upw < 0.0 || target_is_below_node;



        let mut corner_target = port_line_target;

        if !downward {
            corner_target.x = if s1 {
                node_half_width + upward_corner_radius + f32::max(0.0,target.x - node_half_width + upward_corner_radius)
            } else {
                f32::min(node_half_width + (target.x - node_half_width)/2.0,node_half_width + 2.0*upward_corner_radius)
            };
            corner_target.y = f32::min(upward_corner_radius,upw/2.0);
        }


//                if !downward && !s1 {
//                    corner_radius = corner_target.x - node_half_width;
//                    println!("? {}", corner_radius);
//                }


        // === Corner ===

        let corner_grow   = ((corner_target.x - node_half_width) * 0.6).max(0.0);
        let corner_radius = 20.0 + corner_grow;
        let mut corner_radius = corner_radius.min(corner_target.y.abs());
        let corner_x      = corner_target.x - corner_radius;


        //      r1
        //    ◄---►                  (1) x^2 + y^2 = r1^2 + r2^2
        //    _____                  (1) => y = sqrt((r1^2 + r2^2)/x^2)
        //  .'     `.
        // /   _.-"""B-._     ▲
        // | .'0┼    |   `.   |      angle1 = A-XY-0
        // \/   │    /     \  | r2   angle2 = 0-XY-B
        // |`._ │__.'       | |      alpha  = B-XY-X_AXIS
        // |   A└───┼─      | ▼
        // |      (x,y)     |        tg(angle1) = y  / x
        //  \              /         tg(angle2) = r1 / r2
        //   `._        _.'          alpha      = PI - angle1 - angle2
        //      `-....-'


        let x             = (corner_x - node_circle.x).clamp(-corner_radius,node_radius);
        let y             = (node_radius*node_radius + corner_radius*corner_radius - x*x).sqrt();
        let angle1        = f32::atan2(y,x);
        let angle2        = f32::atan2(node_radius,corner_radius);
        let angle_overlap = if corner_x > node_half_width { 0.0 } else { 0.1 };
        let corner_angle  = std::f32::consts::PI - angle1 - angle2;
        let corner_angle  = (corner_angle + angle_overlap) * side;
        let corner_angle  = if downward {corner_angle} else {side * std::f32::consts::PI / 2.0};
        let corner_side   = (corner_radius + PADDING) * 2.0;
        let corner_size   = Vector2::new(corner_side,corner_side);
        let corner_y      = if downward {-y} else {y};
        let start_angle   = if downward {0.0} else {side * std::f32::consts::PI / 2.0};
        let corner        = Vector2::new(corner_x*side,corner_y);

        bg.corner.shape.sprite.size().set(corner_size);
        bg.corner.shape.start_angle.set(start_angle);
        bg.corner.shape.angle.set(corner_angle);
        bg.corner.shape.radius.set(corner_radius);
        bg.corner.shape.pos.set(corner);
        bg.corner.set_position_xy(corner);
        if !target_attached {
            bg.corner.shape.dim.set(Vector2::new(node_half_width,node_radius));
            fg.corner.shape.sprite.size().set(corner_size);
            fg.corner.shape.start_angle.set(start_angle);
            fg.corner.shape.angle.set(corner_angle);
            fg.corner.shape.radius.set(corner_radius);
            fg.corner.shape.pos.set(corner);
            fg.corner.shape.width.set(node_half_width);
            fg.corner.set_position_xy(corner);
        } else {
            fg.corner.shape.sprite.size().set(zero());
            bg.corner.shape.dim.set(Vector2::new(INFINITE,INFINITE));
        }


        // === Side Line ===

        let side_line_len = corner_x - node_half_width;

        if target_attached {
            let bg_line_len  = side_line_len;
            let bg_line_x    = side * (node_half_width + bg_line_len/2.0);
            let bg_line_size = Vector2::new(line_shape_width,bg_line_len + line_sides_overlap);

            fg.side_line.shape.sprite.size().set(zero());
            bg.side_line.shape.sprite.size().set(bg_line_size);
            bg.side_line.mod_position(|p| p.x = bg_line_x);

        } else {
            let bg_line_len  = f32::min(side_line_len,node_padding);
            let fg_line_len  = side_line_len - bg_line_len;
            let bg_line_x    = side * (node_half_width + bg_line_len/2.0);
            let fg_line_x    = side * (node_half_width + fg_line_len/2.0 + bg_line_len + line_side_overlap);
            let bg_line_size = Vector2::new(line_shape_width,bg_line_len + line_sides_overlap * 2.0);
            let fg_line_size = Vector2::new(line_shape_width,fg_line_len);

            bg.side_line.shape.sprite.size().set(bg_line_size);
            bg.side_line.mod_position(|p| p.x = bg_line_x);

            fg.side_line.shape.sprite.size().set(fg_line_size);
            fg.side_line.mod_position(|p| p.x = fg_line_x);
        }



        // === Main Line ===

        let main_line_height   = corner.y - main_line_target.y;
        let main_line_x        = side * port_line_target.x;
        let main_line_y        = corner.y - main_line_height/2.0;
        let main_line_size     = Vector2::new(line_shape_width,main_line_height + line_sides_overlap);
        let main_line_position = Vector3::new(main_line_x,main_line_y,0.0);


        if target_attached {
            fg.main_line.shape.sprite.size().set(zero());
            bg.main_line.shape.sprite.size().set(main_line_size);
            bg.main_line.set_position(main_line_position);
        } else {
            let mut front_line_position = main_line_position;
            let mut front_line_size     = main_line_size;
            let mut back_line_position  = main_line_position;
            let mut back_line_size      = Vector2::new(0.0,0.0);
            let use_double_line         = target_is_below_node;
            if use_double_line {
                let diff               = node_padding - f32::max(0.0,-corner.y - node_radius);
                let diff               = f32::min(diff, -target.y - node_height/2.0);
                front_line_position.y -= diff/2.0;
                front_line_size.y     -= diff;
                back_line_position.y   = corner.y - diff/2.0;
                back_line_size         = main_line_size;
                back_line_size.y       = diff + line_sides_overlap;
            }
            bg.main_line.shape.sprite.size().set(back_line_size);
            bg.main_line.set_position(back_line_position);
            fg.main_line.shape.sprite.size().set(front_line_size);
            fg.main_line.set_position(front_line_position);
        }







        if !downward {


            //
            // ╭─────╮  ╭──╮
            // ╰─────╯──╯  ▢

            // ╭─────╮
            // ╰─────╯────╮
            //            ▢

            // ╭─────╮
            // ╰──┬──╯
            //    ▢

            let corner2_radius = corner_radius;
            let corner3_radius = upward_corner_radius;

            let corner2_x      = corner_target.x + corner_radius;
            let corner3_x      = port_line_target.x - corner3_radius;
            let corner2_bbox_x = corner2_x - corner2_radius;
            let corner3_bbox_x = corner3_x + corner3_radius;

            let corner_2_3_dist     = corner3_bbox_x - corner2_bbox_x;
            let corner_2_3_side     = corner_2_3_dist.signum();
            let corner_2_3_dist     = corner_2_3_dist.abs();
            let corner_2_3_width    = corner2_radius + corner3_radius;
            let corner_2_3_do_scale = corner_2_3_dist < corner_2_3_width;
            let corner_2_3_scale    = corner_2_3_dist / corner_2_3_width;
            let corner_2_3_scale    = if corner_2_3_do_scale {corner_2_3_scale} else {1.0};

            let side_combined       = side * corner_2_3_side;

            let corner2_radius = corner2_radius * corner_2_3_scale;
            let corner3_radius = corner3_radius * corner_2_3_scale;

//////////////////

            let corner3_side   = (corner3_radius + PADDING) * 2.0;
            let corner3_size   = Vector2::new(corner3_side,corner3_side);
            let corner3_x      = port_line_target.x - corner_2_3_side * corner3_radius;
            let corner3_y      = port_line_target.y;

            let corner2_y      = corner3_y + corner3_radius - corner2_radius;
            let corner2_y      = f32::max(corner2_y, corner_y);

            let corner3_y      = f32::max(corner3_y,corner2_y - corner3_radius + corner2_radius);

            let corner3        = Vector2::new(corner3_x*side,corner3_y);
            let corner3_angle  = if (side_combined == 1.0) {0.0} else {-std::f32::consts::PI / 2.0};

            fg.corner3.shape.sprite.size().set(corner3_size);
            fg.corner3.shape.start_angle.set(corner3_angle);
            fg.corner3.shape.angle.set(std::f32::consts::PI / 2.0);
            fg.corner3.shape.radius.set(corner3_radius);
            fg.corner3.shape.pos.set(corner3);
            fg.corner3.shape.width.set(0.0);
            fg.corner3.set_position_xy(corner3);

            port_line_len = corner3_y - target.y;

            let xoff = 20.0;

            let corner2_x      = corner_target.x + corner_2_3_side * corner2_radius;

            let corner2        = Vector2::new(corner2_x*side,corner2_y);
            let corner2_angle  = if (side_combined == 1.0) {-std::f32::consts::PI / 2.0} else {0.0};

            fg.corner2.shape.sprite.size().set(corner_size);
            fg.corner2.shape.start_angle.set(corner2_angle);
            fg.corner2.shape.angle.set(std::f32::consts::PI / 2.0);
            fg.corner2.shape.radius.set(corner2_radius);
            fg.corner2.shape.pos.set(corner2);
            fg.corner2.shape.width.set(0.0);
            fg.corner2.set_position_xy(corner2);






            let main_line_len    = corner2_y - corner_y;
            let main_line_size   = Vector2::new(line_shape_width,main_line_len + line_sides_overlap);
            let main_line_x      = side * corner_target.x;
            let main_line_y      = main_line_len / 2.0 + corner_y;
            let main_line_pos    = Vector2::new(main_line_x,main_line_y);

            fg.main_line.shape.sprite.size().set(main_line_size);
            fg.main_line.set_position_xy(main_line_pos);

            if main_line_len > 0.0 {
                let arrow_pos = Vector2::new(main_line_pos.x, (corner_y - corner_radius + corner2_y + corner2_radius)/2.0);
                fg.arrow.shape.sprite.size().set(Vector2::new(20.0,20.0));
                fg.arrow.set_position_xy(arrow_pos);
            } else {
                fg.arrow.shape.sprite.size().set(zero());
            }

            println!(">> {}",main_line_len);

            let side_line2_len  = corner3_x - corner2_x;
            let side_line2_x    = side * (corner2_x + side_line2_len / 2.0);
            let side_line2_y    = corner2_y + corner2_radius;
            let side_line2_len  = side_line2_len.abs();
            let side_line2_pos  = Vector2::new(side_line2_x,side_line2_y);
            let side_line2_size = Vector2::new(line_shape_width,side_line2_len + line_sides_overlap);

            fg.side_line2.shape.sprite.size().set(side_line2_size);
            fg.side_line2.set_position_xy(side_line2_pos);

        } else {
            fg.corner3.shape.sprite.size().set(zero());
            fg.corner2.shape.sprite.size().set(zero());
            fg.side_line2.shape.sprite.size().set(zero());
        }

        // === Port Line ===

        fg.port_line.shape.sprite.size().set(Vector2::new(line_shape_width,port_line_len-END_OFFSET));
        fg.port_line.mod_position(|p| {
            p.x = main_line_x;
            p.y = port_line_target.y - port_line_len_max + port_line_len/2.0 + END_OFFSET;
        });
    }
}

impl Edge {
    /// Constructor.
    pub fn new(scene:&Scene) -> Self {
        let logger    = Logger::new("edge");
        let display_object    = display::object::Instance::new(&logger);
        let fg     = Front::new(logger.sub("fg"),scene);
        let bg      = Back::new(logger.sub("bg"),scene);

        display_object.add_child(&fg);
        display_object.add_child(&bg);

        fg.side_line.mod_rotation(|r| r.z = std::f32::consts::PI/2.0);
        bg.side_line.mod_rotation(|r| r.z = std::f32::consts::PI/2.0);
        fg.side_line2.mod_rotation(|r| r.z = std::f32::consts::PI/2.0);
        bg.side_line2.mod_rotation(|r| r.z = std::f32::consts::PI/2.0);

        let network = frp::Network::new();
        let input = Frp::new(&network);

        let source_width : Rc<Cell<f32>> = default();
        let target_position = Rc::new(Cell::new(frp::Position::default()));
        source_width.set(100.0);

        let target_attached : Rc<Cell<bool>> = default();







        let frp = input;
        let front = fg;
        let back  = bg;
        let data = Rc::new(EdgeModelData {display_object,logger,frp,front,back
                                          ,source_width,target_position,target_attached});
        let model = Rc::new(EdgeModel {data});




        Self {model,network} . init()
    }

    fn init(self) -> Self {
        let network         = &self.network;
        let input           = &self.frp;
        let target_position = &self.target_position;
        let target_attached = &self.target_attached;
        let source_width    = &self.source_width;
        let model           = &self.model;
        frp::extend! { network
            eval input.target_position ((t) target_position.set(*t));
            eval input.target_attached ((t) target_attached.set(*t));
            eval input.source_width    ((t) source_width.set(*t));
            on_change <- any_ (input.source_width, input.target_position, input.target_attached);
            eval_ on_change (model.redraw());
        }
        self
    }
}

impl display::Object for Edge {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
