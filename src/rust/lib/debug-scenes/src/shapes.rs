#![allow(missing_docs)]

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

use ensogl::prelude::*;

use ensogl::display::navigation::navigator::Navigator;
use ensogl::system::web;
use ensogl::application::Application;
use graph_editor::GraphEditor;
use wasm_bindgen::prelude::*;
use ensogl::display::object::ObjectOps;
use ensogl_core_msdf_sys::run_once_initialized;
use ensogl::display::style::theme;
use ensogl::data::color;
use enso_frp as frp;
use ensogl_text as text;

use text::traits::*;

use nalgebra as n;


#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_shapes() {
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


fn init(app:&Application) {

    let mut dark = theme::Theme::new();
    dark.insert("application.background.color", color::Lcha::new(0.13,0.013,0.18,1.0));
    dark.insert("graph_editor.node.background.color", color::Lcha::new(0.2,0.013,0.18,1.0));
    dark.insert("graph_editor.node.selection.color", color::Lcha::new(0.72,0.5,0.22,1.0));
    dark.insert("graph_editor.node.selection.size", 7.0);
    dark.insert("animation.duration", 0.5);
    dark.insert("graph.node.shadow.color", 5.0);
    dark.insert("graph.node.shadow.size", 5.0);
    dark.insert("mouse.pointer.color", color::Rgba::new(0.3,0.3,0.3,1.0));

    app.themes.register("dark",dark);
    app.themes.set_enabled(&["dark"]);

    let _bg = app.display.scene().style_sheet.var("application.background.color");



    let world     = &app.display;
    let scene     = world.scene();
    let camera    = scene.camera();
    let navigator = Navigator::new(&scene,&camera);

    app.views.register::<GraphEditor>();
    let graph_editor = app.new_view::<GraphEditor>();
    world.add_child(&graph_editor);


    let node1_id = graph_editor.add_node();
    let node2_id = graph_editor.add_node();
//
    graph_editor.frp.set_node_position.emit((node1_id,Vector2(-150.0,50.0)));
    graph_editor.frp.set_node_position.emit((node2_id,Vector2(50.0,50.0)));
//
    graph_editor.frp.set_node_expression.emit((node1_id,expression_mock()));
    graph_editor.frp.set_node_expression.emit((node2_id,expression_mock2()));

//    frp::new_network! { network
//        def trigger = source::<()>();
//        let (runner,condition) = fence(&network,&trigger);
//        def _eval = runner.map(f_!( {
//            graph_editor.frp.connect_nodes.emit((EdgeTarget::new(node1_id,default()),EdgeTarget::new(node2_id,vec![1,0,2])));
//        }));
//        def _debug = graph_editor.frp.outputs.edge_added.map2(&condition, |id,cond| {
//            let owner = if *cond { "GUI" } else { "ME" };
//            println!("Edge {:?} added by {}!",id,owner)
//        });
//
//    }

//    trigger.emit(());


//    let fonts         = text::typeface::font::Registry::init_and_load_default();
//    let font          = fonts.load("DejaVuSansMono");
//    let glyph_system  = text::typeface::glyph::System::new(scene,font);
//    let symbol        = &glyph_system.sprite_system().symbol;
//    scene.views.main.remove(symbol);
//    scene.views.label.add(symbol);

//    let buffer = text::Buffer::from("Test text €!!!\nline2\nline3\nline4");
//    let buffer_view = buffer.new_view();
//
////    buffer.set((..),color::Rgba::new(0.8,0.8,0.8,1.0));
//
//
////    buffer.set((..),color::Rgba::new(0.8,0.8,0.8,1.0));
//    buffer.set((1..3).bytes(),color::Rgba::new(0.0,1.0,0.0,1.0));
//    buffer.set((8..9).bytes(),color::Rgba::new(1.0,1.0,0.0,1.0));
//    buffer.set((10..11).bytes(),color::Rgba::new(1.0,0.0,0.0,1.0));
//    buffer.set((14..15).bytes(),color::Rgba::new(0.0,0.0,1.0,1.0));
//
//    buffer.set_default(color::Rgba::new(0.8,0.8,0.8,1.0));
//    buffer.set_default(text::Size(10.0));
//    buffer.set((0..4).bytes(),text::Size(20.0));




    let area = text::Area::new(Logger::new("test"),scene);
    world.add_child(&area);

    area.add_cursor(0.bytes());
//    area.insert("Test text €!!!\nline2\nline3\nopen \"data.csv\"");
    area.insert("open€ \"data.csv\"");


    area.set((1..3).bytes(),color::Rgba::new(0.0,1.0,0.0,1.0));
    area.set((8..9).bytes(),color::Rgba::new(1.0,1.0,0.0,1.0));
    area.set((10..11).bytes(),color::Rgba::new(1.0,0.0,0.0,1.0));
    area.set((14..15).bytes(),color::Rgba::new(0.0,0.0,1.0,1.0));

    area.set_default(color::Rgba::new(1.0,1.0,1.0,0.7));
    area.set_default(text::Size(12.0));
    area.set_position_x(10.0);
//    area.set((0..4).bytes(),text::Size(20.0));

    area.insert("!!!!");
    area.undo();

    area.redraw();

    let cursor = &app.cursor;

    frp::new_network! { network
        eval area.frp.output.mouse_cursor_style ((s) cursor.frp.input.set_style.emit(s));
    }

//    area.tmp_replace_all_text("Test text €!!!\nline2\nline3\nline4");

//    println!("!!! {}", buffer.text.rope.subseq(0..t10));
//    println!("!!! {:?}", buffer_view.offset_of_view_line(text::Line(0)));
//    println!("!!! {:?}", buffer_view.offset_of_view_line(text::Line(1)));
//    println!("!!! {:?}", buffer_view.offset_of_view_line(text::Line(2)));
//    println!("!!! {:?}", buffer_view.offset_of_view_line(text::Line(3)));

    let object_space  = n::Vector4::new(3.0,5.0,7.0,1.0);
    let object_matrix = n::Matrix4::identity().append_translation(&n::Vector3::new(10.0,0.0,0.0));
    let inv_object_matrix = object_matrix.try_inverse().unwrap();

    let camera_matrix   = n::Matrix4::identity().append_translation(&n::Vector3::new(0.0,0.0,100.0));
    let inv_view_matrix = camera_matrix;
    let view_matrix     = camera_matrix.try_inverse().unwrap();

    let aspect = 1.0;
    let fov    = 90.0f32.to_radians();
    let near   = 0.1;
    let far    = 100.0;
    let perspective           = n::Perspective3::new(aspect,fov,near,far);
    let projection_matrix     = *perspective.as_matrix();
    let inv_projection_matrix = perspective.inverse();

    // let left   = -100.0;
    // let right  = 100.0;
    // let bottom = -100.0;
    // let top    = 100.0;
    // let near   = 0.0;
    // let far    = 100.0;
    // let orthographic          = n::Orthographic3::new(left,right,bottom,top,near,far);
    // let projection_matrix     = *orthographic.as_matrix();
    // let inv_projection_matrix = orthographic.inverse();

    let world_space   = object_matrix * object_space;
    let eye_space     = view_matrix * world_space;
    let clip_space    = projection_matrix * eye_space;

    let eye_space2    = inv_projection_matrix * clip_space;
    let world_space2  = inv_view_matrix * eye_space2;
    let object_space2 = inv_object_matrix * world_space2;

    println!("---------------------------------");
    println!("object_space: {:?}", object_space);
    println!("world_space: {:?}", world_space);
    println!("eye_space: {:?}", eye_space);
    println!("clip_space: {:?}", clip_space);
    println!("eye_space2: {:?}", eye_space2);
    println!("world_space2: {:?}", world_space2);
    println!("object_space2: {:?}", object_space2);


    let mut was_rendered = false;
    let mut loader_hidden = false;
    world.on_frame(move |_| {
        let _keep_alive = &navigator;
        let _keep_alive = &graph_editor;
        let _keep_alive = &area;
        let _keep_alive = &network;

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

use ast::crumbs::PatternMatchCrumb::*;
use ast::crumbs::*;
use span_tree::traits::*;
use graph_editor::component::node::port::Expression;


pub fn expression_mock() -> Expression {
    let code             = "open \"data.csv\"".into();
    let output_span_tree = default();
    let input_span_tree  = span_tree::builder::TreeBuilder::new(15)
        .add_leaf(0,4,span_tree::node::Kind::Operation,PrefixCrumb::Func)
        .add_empty_child(5,span_tree::node::InsertType::BeforeTarget)
        .add_leaf(5,10,span_tree::node::Kind::Target{is_removable:false},PrefixCrumb::Arg)
        .add_empty_child(15,span_tree::node::InsertType::Append)
        .build();
    Expression {code,input_span_tree,output_span_tree}
}

pub fn expression_mock2() -> Expression {
    let pattern_cr       = vec![Seq { right: false }, Or, Or, Build];
    let val              = ast::crumbs::SegmentMatchCrumb::Body {val:pattern_cr};
    let parens_cr        = ast::crumbs::MatchCrumb::Segs {val,index:0};
    let code             = "make_maps size (distribution normal)".into();
    let output_span_tree = default();
    let input_span_tree  = span_tree::builder::TreeBuilder::new(36)
        .add_child(0,14,span_tree::node::Kind::Chained,PrefixCrumb::Func)
        .add_leaf(0,9,span_tree::node::Kind::Operation,PrefixCrumb::Func)
        .add_empty_child(10,span_tree::node::InsertType::BeforeTarget)
        .add_leaf(10,4,span_tree::node::Kind::Target {is_removable:true},PrefixCrumb::Arg)
        .add_empty_child(14,span_tree::node::InsertType::Append)
        .done()
        .add_child(15,21,span_tree::node::Kind::Argument {is_removable:true},PrefixCrumb::Arg)
        .add_child(1,19,span_tree::node::Kind::Argument {is_removable:false},parens_cr)
        .add_leaf(0,12,span_tree::node::Kind::Operation,PrefixCrumb::Func)
        .add_empty_child(13,span_tree::node::InsertType::BeforeTarget)
        .add_leaf(13,6,span_tree::node::Kind::Target {is_removable:false},PrefixCrumb::Arg)
        .add_empty_child(19,span_tree::node::InsertType::Append)
        .done()
        .done()
        .add_empty_child(36,span_tree::node::InsertType::Append)
        .build();
    Expression {code,input_span_tree,output_span_tree}
}



// TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO
// TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO
// TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO

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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_with_no_rules() {
        assert_eq!( depth_sort(&vec![]      , &default()) , Vec::<usize>::new() );
        assert_eq!( depth_sort(&vec![1]     , &default()) , vec![1] );
        assert_eq!( depth_sort(&vec![1,3]   , &default()) , vec![1,3] );
        assert_eq!( depth_sort(&vec![1,2,3] , &default()) , vec![1,2,3] );
    }


    #[test]
    fn chained_rules() {
        let mut rules = HashMap::<usize,Vec<usize>>::new();
        rules.insert(1,vec![2]);
        rules.insert(2,vec![3]);
        assert_eq!( depth_sort(&vec![]      , &rules) , Vec::<usize>::new() );
        assert_eq!( depth_sort(&vec![1]     , &rules) , vec![1] );
        assert_eq!( depth_sort(&vec![1,2]   , &rules) , vec![2,1] );
        assert_eq!( depth_sort(&vec![1,2,3] , &rules) , vec![3,2,1] );
    }

    #[test]
    fn order_preserving() {
        let mut rules = HashMap::<usize,Vec<usize>>::new();
        rules.insert(1,vec![2]);
        rules.insert(2,vec![3]);
        assert_eq!( depth_sort(&vec![10,11,12]          , &rules) , vec![10,11,12] );
        assert_eq!( depth_sort(&vec![10,1,11,12]        , &rules) , vec![10,1,11,12] );
        assert_eq!( depth_sort(&vec![10,1,11,2,12]      , &rules) , vec![10,11,2,1,12] );
        assert_eq!( depth_sort(&vec![10,1,11,2,12,3,13] , &rules) , vec![10,11,12,3,2,1,13] );
    }
}
