#![allow(missing_docs)]

//! NOTE
//! This file illustrates a bug with the shape parenting system. Look for the
//! "Buggy behaviour" section to find the problematic code.

use crate::prelude::*;

use ensogl_core_msdf_sys::run_once_initialized;
use wasm_bindgen::prelude::*;

use crate::graph_editor::GraphEditor;

use enso_frp as frp;
use ensogl::display::navigation::navigator::Navigator;
use ensogl::system::web;
use ensogl::application::Application;
use ensogl::display::object::ObjectOps;
use ensogl::data::color;
use ensogl::gui::component;
use ensogl::display;
use ensogl::display::shape::*;
use ensogl::display::scene::Scene;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Sprite;




// ====================
// === Dummy Shapes ===
// ====================

pub mod shape {
    use super::*;

    ensogl::define_shape_system! {
        (color_rgba:Vector4<f32>) {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let shape  = Rect((&width,&height));
            let color  : Var<color::Rgba> = color_rgba.into();
            let shape  = shape.fill(color);
            shape.into()
        }
    }
}

#[derive(Clone,CloneRef,Debug)]
struct DummyShape {
    display_object : display::object::Instance,
    shape          : component::ShapeView<shape::Shape>,
}

impl DummyShape {
    fn new(scene:&Scene) -> Self {
        let logger         = Logger::new("dummy_shape");
        let display_object = display::object::Instance::new(&logger);
        let shape          = component::ShapeView::<shape::Shape>::new(&logger,&scene);
        display_object.add_child(&shape);
        shape.shape.sprite.size.set(Vector2::new(50.0,50.0));
        DummyShape{display_object,shape}
    }
}

impl display::Object for DummyShape {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_parenting() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();
    run_once_initialized(|| {
        let app = Application::new(&web::get_html_element_by_id("root").unwrap());
        init(&app);
        mem::forget(app);
    });
}


fn _fence<T,Out>(network:&frp::Network, trigger:T) -> (frp::Stream,frp::Stream<bool>)
    where T:frp::HasOutput<Output=Out>, T:Into<frp::Stream<Out>>, Out:frp::Data {
    let trigger = trigger.into();
    frp::extend! { network
        def trigger_ = trigger.constant(());
        def runner   = source::<()>();
        def switch   = any_mut();
        switch.attach(&trigger_);
        def triggered = trigger.map(f_!(runner.emit(())));
        switch.attach(&triggered);
        def condition = switch.toggle_true();
    }
    let runner = runner.into();
    (runner,condition)
}



// ========================
// === Init Application ===
// ========================

fn init(app:&Application) {
    let world     = &app.display;
    let scene     = world.scene();
    let camera    = scene.camera();
    let navigator = Navigator::new(&scene,&camera);

    app.views.register::<GraphEditor>();
    let graph_editor = app.new_view::<GraphEditor>();
    world.add_child(&graph_editor);

    // === Shape setup ===

    let mut dummies = Vec::default();

    let node1_id = graph_editor.add_node();
    let node = graph_editor.nodes.get_cloned(&node1_id).unwrap();

    let root  = DummyShape::new(scene);
    root.shape.shape.color_rgba.set(Vector4::new(1.0,0.0,0.0,0.5));
    root.set_position_xy(Vector2::new(0.0,-150.0));
    node.add_child(&root);

    let child_1  = DummyShape::new(scene);
    child_1.shape.shape.color_rgba.set(Vector4::new(0.0,1.0,0.0,0.5));
    child_1.set_position_xy(Vector2::new(0.0,-150.0));

    let child_2  = DummyShape::new(scene);
    child_2.shape.shape.color_rgba.set(Vector4::new(0.0,0.0,1.0,0.5));
    child_2.set_position_xy(Vector2::new(0.0,-150.0));

    // ==========================================
    // === Buggy behaviour activation start =====
                                             //==
    root   .add_child(&child_1);             //==
    child_1.add_child(&child_2);             //==
    child_2.unset_parent();                  //==
    child_1.unset_parent();                  //==
                                             //==
    // === Buggy behaviour activation end =======
    // ==========================================

    // It seems there is no parent any more.
    debug_assert!(!child_1.has_parent());
    debug_assert!(!child_2.has_parent());

    // Neither shape should be visible. But the scene contains a blue shape (`child_2`) stuck to
    // the center. Moving the node will not move the shape.

    // Move the node so this is easier to see without interaction.
    node.set_position_xy(Vector2(200.0,200.0));

    dummies.push(root);
    dummies.push(child_1);
    dummies.push(child_2);

    frp::new_network! { network
    }

    let mut was_rendered = false;
    let mut loader_hidden = false;
    world.on_frame(move |_| {
        let _keep_alive = &navigator;
        let _keep_alive = &graph_editor;
        let _keep_alive = &network;
        let _keep_alive = &dummies;

        // Temporary code removing the web-loader instance.
        // To be changed in the future.
        if was_rendered && !loader_hidden {
            web::get_element_by_id("loader").map(|t| {
                t.parent_node().map(|p| {
                    p.remove_child(&t).unwrap()
                })
            }).ok();
            loader_hidden = true;
        }
        was_rendered = true;
    }).forget();

}



// =============
// === Mocks ===
// =============

// Extract and make use in scene depth sorting.

#[allow(clippy::implicit_hasher)]
pub fn depth_sort(ids:&[usize], elem_above_elems:&HashMap<usize,Vec<usize>>) -> Vec<usize> {

    // === Remove from `elem_above_elems` all ids which are not present in `ids` ===

    let mut elem_above_elems : HashMap<usize,Vec<usize>> = elem_above_elems.clone();
    let mut missing = vec![];
    for (elem,above_elems) in &mut elem_above_elems {
        above_elems.retain(|id| ids.contains(id));
        if above_elems.is_empty() {
            missing.push(*elem);
        }
    }
    for id in &missing {
        elem_above_elems.remove(id);
    }


    // === Generate `elem_below_elems` map ===

    let mut elem_below_elems : HashMap<usize,Vec<usize>> = HashMap::new();
    for (above_id,below_ids) in &elem_above_elems {
        for below_id in below_ids {
            elem_below_elems.entry(*below_id).or_default().push(*above_id);
        }
    }


    // === Sort ids ===

    let mut queue        = HashSet::<usize>::new();
    let mut sorted       = vec![];
    let mut newly_sorted = vec![];

    for id in ids {
        if elem_above_elems.get(id).is_some() {
            queue.insert(*id);
        } else {
            newly_sorted.push(*id);
            while !newly_sorted.is_empty() {
                let id = newly_sorted.pop().unwrap();
                sorted.push(id);
                elem_below_elems.remove(&id).for_each(|above_ids| {
                    for above_id in above_ids {
                        if let Some(lst) = elem_above_elems.get_mut(&above_id) {
                            lst.remove_item(&id);
                            if lst.is_empty() && queue.contains(&above_id) {
                                queue.remove(&above_id);
                                newly_sorted.push(above_id);
                            }
                            if lst.is_empty() {
                                elem_above_elems.remove(&above_id);
                            }
                        }
                    }
                })
            }
        }
    }
    sorted
}
