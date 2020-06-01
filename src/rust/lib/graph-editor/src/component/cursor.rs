//! Definition of the Cursor (known as well as mouse pointer) component.

#![allow(missing_docs)]
// WARNING! UNDER HEAVY DEVELOPMENT. EXPECT DRASTIC CHANGES.

use crate::prelude::*;

use enso_frp as frp;
use ensogl::control::callback;
use ensogl::data::color;
use ensogl::display::Buffer;
use ensogl::display::layout::alignment;
use ensogl::display::scene::Scene;
use ensogl::display::shape::*;
use ensogl::display::{Sprite, Attribute};
use ensogl::display;
use ensogl::gui::component::animation;
use ensogl::gui::component::animation2;
use ensogl::gui::component::animator;
use ensogl::gui::component::Animator;
use ensogl::gui::component::animator2;
use ensogl::gui::component;
use ensogl::system::web;
use ensogl::animation::physics::inertia::Spring;

use ensogl::display::object::class::ObjectOps; // FIXME: why?


// ==================
// === StyleParam ===
// ==================

#[derive(Debug,Clone)]
pub struct StyleParam<T> {
    pub value   : T,
    pub animate : bool,
}

impl<T:Default> Default for StyleParam<T> {
    fn default() -> Self {
        let value   = default();
        let animate = true;
        Self {value,animate}
    }
}

impl<T> StyleParam<T> {
    pub fn new(value:T) -> Self {
        let animate = true;
        Self {value,animate}
    }

    pub fn new_no_animation(value:T) -> Self {
        let animate = false;
        Self {value,animate}
    }
}



// =============
// === Style ===
// =============

#[derive(Debug,Clone,Default)]
pub struct Style {
    host   : Option<display::object::Instance>,
    size   : Option<StyleParam<Vector2<f32>>>,
    offset : Option<StyleParam<Vector2<f32>>>,
    color  : Option<StyleParam<color::Lcha>>,
    radius : Option<f32>,
    press  : Option<f32>,
}

impl Style {
    pub fn highlight<H>
    (host:H, size:Vector2<f32>, color:Option<color::Lcha>) -> Self
    where H:display::Object {
        let host  = Some(host.display_object().clone_ref());
        let size  = Some(StyleParam::new(size));
        let color = color.map(StyleParam::new);
        Self {host,size,color,..default()}
    }

    pub fn color(color:color::Lcha) -> Self {
        let color = Some(StyleParam::new(color));
        Self {color,..default()}
    }

    pub fn color_no_animation(color:color::Lcha) -> Self {
        let color = Some(StyleParam::new_no_animation(color));
        Self {color,..default()}
    }

    pub fn selection(size:Vector2<f32>) -> Self {
        let offset = Some(StyleParam::new_no_animation(-size / 2.0));
        let size   = Some(StyleParam::new_no_animation(size.abs() + Vector2::new(16.0,16.0)));
        Self {size,offset,..default()}
    }

    pub fn pressed() -> Self {
        let press = Some(1.0);
        Self {press,..default()}
    }

    pub fn press(mut self) -> Self {
        self.press = Some(1.0);
        self
    }
}

impl PartialSemigroup<&Style> for Style {
    #[allow(clippy::clone_on_copy)]
    fn concat_mut(&mut self, other:&Self) {
        if self.host   . is_none() { self.host   = other.host   . clone() }
        if self.size   . is_none() { self.size   = other.size   . clone() }
        if self.offset . is_none() { self.offset = other.offset . clone() }
        if self.color  . is_none() { self.color  = other.color  . clone() }
        if self.radius . is_none() { self.radius = other.radius . clone() }
        if self.press  . is_none() { self.press  = other.press  . clone() }
    }
}

impl PartialSemigroup<Style> for Style {
    fn concat_mut(&mut self, other:Self) {
        self.concat_mut(&other)
    }
}



// ==================
// === CursorView ===
// ==================

/// Canvas shape definition.
pub mod shape {
    use super::*;

    ensogl::define_shape_system! {
        ( offset         : V2
        , selection_size : Vector2<f32>
        , press          : f32
        , radius         : f32
        , color          : Vector4<f32>
        ) {
            let width  : Var<Distance<Pixels>> = "input_size.x".into();
            let height : Var<Distance<Pixels>> = "input_size.y".into();
            let press_diff       = 2.px() * &press;
            let radius           = 1.px() * radius - &press_diff;
            let offset : Var<V2<Distance<Pixels>>>           = offset.px();
            let selection_width  = 1.px() * &selection_size.x(); // * &press;
            let selection_height = 1.px() * &selection_size.y(); // * &press;
            let width            = (&width  - &press_diff * 2.0) + selection_width.abs();
            let height           = (&height - &press_diff * 2.0) + selection_height.abs();
            let cursor = Rect((width,height))
                .corners_radius(radius)
                //.translate(offset)
                .fill("srgba(input_color)");
            cursor.into()
        }
    }
}



// =================
// === FrpInputs ===
// =================

/// Cursor events.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct FrpInputs {
    pub set_style : frp::Source<Style>,
}

impl FrpInputs {
    /// Constructor.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! { network
            def set_style = source();
        }
        Self {set_style}
    }
}

/// Cursor events.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Frp {
    pub network  : frp::Network,
    pub input    : FrpInputs,
    pub position : frp::Stream<V3>,
}

impl Deref for Frp {
    type Target = FrpInputs;
    fn deref(&self) -> &Self::Target {
        &self.input
    }
}



// ==============
// === Cursor ===
// ==============

/// Cursor (mouse pointer) definition.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct Cursor {
    #[shrinkwrap(main_field)]
    model   : Rc<CursorModel>,
    pub frp : Frp,
}



/// Internal data for `Cursor`.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct CursorModel {
    pub logger : Logger,
    pub scene  : Scene,
    pub frp    : FrpInputs,
    pub view   : component::ShapeView<shape::Shape>,
    pub style  : Rc<RefCell<Style>>,
}

impl CursorModel {
    pub fn new(scene:&Scene, network:&frp::Network) -> Self {
        let logger = Logger::new("cursor");
        let frp    = FrpInputs::new(network);
        let view   = component::ShapeView::<shape::Shape>::new(&logger,scene);
        let scene_shape = scene.shape();
        let shape  = &view.shape;
        shape.sprite.size().set(Vector2::new(50.0,50.0));
//        let resize_handle = scene.on_resize(enclose!((shape) move |scene_shape:&web::dom::ShapeData| {
//            shape.sprite.size().set(Vector2::new(scene_shape.width(),scene_shape.height()));
//        }));
        let style = Rc::new(RefCell::new(Style::default()));

        let shape_system = scene.shapes.shape_system(PhantomData::<shape::Shape>);
//        shape_system.shape_system.set_alignment(alignment::HorizontalAlignment::Left, alignment::VerticalAlignment::Bottom);
        shape_system.shape_system.set_pointer_events(false);

        scene.views.main.remove(&shape_system.shape_system.symbol);
        scene.views.cursor.add(&shape_system.shape_system.symbol);

        let scene = scene.clone_ref();

        Self {logger,scene,frp,view,style}
    }
}

impl Cursor {
    /// Constructor.
    pub fn new(scene:&Scene) -> Self {
        let network = frp::Network::new();

        let model = CursorModel::new(scene,&network);

        let view = &model.view;
        let input = &model.frp;
        let style = &model.style;


        let view_data = view.shape.clone_ref();
        let press = animation(&network,move |value| {
            view_data.press.set(value)
        });

        let view_data = view.shape.clone_ref();
        let radius = animation(&network,move |value| {
            view_data.radius.set(value)
        });

        let size                 = Animator::<V2>::new(&network);
        let offset               = Animator::<V2>::new(&network);
        let host_position        = Animator::<V3>::new(&network);
        let host_position_weight = Animator::<f32>::new(&network);
        let (host_position_snap,current_host_position_snap) = animator2(&network);
        host_position_snap.set_duration(300.0);

        let (anim_color_lab_l_setter,anim_color_lab_l) = animation2(&network);
        let (anim_color_lab_a_setter,anim_color_lab_a) = animation2(&network);
        let (anim_color_lab_b_setter,anim_color_lab_b) = animation2(&network);
        let (anim_color_alpha_setter,anim_color_alpha) = animation2(&network);


        anim_color_lab_l_setter.set_target_value(1.0);
        anim_color_alpha_setter.set_target_value(0.2);

        radius.set_target_value(8.0);
        size.set_target_value(V2::new(16.0,16.0));

        let mouse = &scene.mouse.frp;




        frp::extend! { network

            eval size.value ((v) view.shape.sprite.size().set(Vector2::new(v.x,v.y)));
            eval offset.value ((v) view.shape.offset.set(*v));

//            def anim_position = anim_pos_x.all_with(&anim_pos_y,|x,y| frp::Position::new(*x,*y));

            anim_color <- all_with4(&anim_color_lab_l,&anim_color_lab_a,&anim_color_lab_b,&anim_color_alpha,
                |l,a,b,alpha| color::Rgba::from(color::Laba::new(*l,*a,*b,*alpha))
            );



            def _ev = input.set_style.map(enclose!((host_position_weight,style,size,host_position) move |new_style| {
//                host_position_snap.set_target_value(0.0);
//                host_position_snap.skip();
                host_position_snap.rewind();

                match &new_style.press {
                    None    => press.set_target_value(0.0),
                    Some(t) => press.set_target_value(*t),
                }

                match &new_style.host {
                    Some(_) => host_position_snap.start(), // set_target_value(100.0),
                    _ => {}
                }
//                match &new_style.host {
//                    None       => host_position_weight.set_target_value(0.0),
//                    Some(host) => {
//                        let position = host.global_position();
//                        host_position.set_target_value(V2(position.x,position.y));
//                        host_position_weight.set_target_value(1.0);
//                    }
//                }

                match &new_style.color {
                    None => {
                        anim_color_lab_l_setter.set_target_value(1.0);
                        anim_color_lab_a_setter.set_target_value(0.0);
                        anim_color_lab_b_setter.set_target_value(0.0);
                        anim_color_alpha_setter.set_target_value(0.2);
                    }
                    Some(new_color) => {
                        let lab = color::Laba::from(new_color.value);
                        anim_color_lab_l_setter.set_target_value(lab.lightness);
                        anim_color_lab_a_setter.set_target_value(lab.a);
                        anim_color_lab_b_setter.set_target_value(lab.b);
                        anim_color_alpha_setter.set_target_value(lab.alpha);
                        if !new_color.animate {
                            anim_color_lab_l_setter.skip();
                            anim_color_lab_a_setter.skip();
                            anim_color_lab_b_setter.skip();
                            anim_color_alpha_setter.skip();
                        }
                    }
                }

                match &new_style.size {
                    None => {
                        size.set_target_value(V2::new(16.0,16.0));
                    }
                    Some(new_size) => {
                        size.set_target_value(V2::new(new_size.value.x,new_size.value.y));
                        if !new_size.animate { size.skip() }
                    }
                }

                match &new_style.offset {
                    None => {
                        offset.set_target_value(V2::new(0.0,0.0));
                    }
                    Some(new_offset) => {
                        offset.set_target_value(V2::new(new_offset.value.x,new_offset.value.y));
                        if !new_offset.animate { offset.skip() }
                    }
                }

                match &new_style.radius {
                    None    => radius.set_target_value(8.0),
                    Some(r) => radius.set_target_value(*r),
                }



                *style.borrow_mut() = new_style.clone();
            }));



            host_changed <- any_(input.set_style,scene.frp.camera_changed);

            def hosted_position = host_changed.map(f_!(model.style.borrow().host.as_ref().map(|t| t.global_position())));

            def is_not_hosted = hosted_position.map(|p| p.is_none());
            def mouse_position_not_hosted = mouse.position.gate(&is_not_hosted);


            eval_ host_changed([model,host_position,host_position_weight] {
                match &model.style.borrow().host {
                    None       => {
                        host_position_weight.set_target_value(0.0);
                        let z = model.scene.views.cursor.camera.z_zoom_1();
                    }
                    Some(host) => {
                        host_position_weight.set_target_value(1.0);
                        let m1 = model.scene.views.cursor.camera.inversed_view_matrix();
                        let m2 = model.scene.camera().view_matrix();

                        let position    = host.global_position();
                        let position    = Vector4::new(position.x,position.y,position.z,1.0);
                        let position    = m2 * (m1 * position);
                        host_position.set_target_value(V3(position.x,position.y,position.z));
                    }
                }
            });


            hp <- host_changed.all_with3(&current_host_position_snap,&host_position.value, f!([host_position](_,s,p) {
                let tp = host_position.target_value();
                let x  = s * tp.x + (1.0 - s) * p.x;
                let y  = s * tp.y + (1.0 - s) * p.y;
                let z  = s * tp.z + (1.0 - s) * p.z;
                V3(x,y,z)
            }));

            def position = mouse.position.all_with3(&hp,&host_position_weight.value, |p,ap,au| {
                let x = ap.x * au + p.x * (1.0 - au);
                let y = ap.y * au + p.y * (1.0 - au);
                let z = ap.z * au;// + p.z * (1.0 - au);
                V3(x,y,z)
            });

            eval anim_color    ((t) view.shape.color.set(Vector4::new(t.red,t.green,t.blue,t.alpha)));
            eval position ((p) view.set_position(Vector3::new(p.x,p.y,p.z)));

            def _position = mouse_position_not_hosted.map(f!([host_position](p) {
                host_position.set_target_value(V3(p.x,p.y,0.0));
            }));
        }





        input.set_style.emit(Style::default());
        let input = input.clone_ref();

        let model   = Rc::new(model);

        let frp    = Frp {network,input,position};

        Cursor {frp,model}
    }
}

impl display::Object for Cursor {
    fn display_object(&self) -> &display::object::Instance {
        &self.view.display_object
    }
}
