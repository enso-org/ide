#![allow(missing_docs)]

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

use crate::prelude::*;

use crate::graph_editor;
use crate::graph_editor::GraphEditor;
use crate::graph_editor::NodeProfilingStatus;
use crate::graph_editor::Type;
use crate::project;
use crate::status_bar;

use enso_frp as frp;
use ensogl::display::navigation::navigator::Navigator;
use ensogl::system::web;
use ensogl::application::Application;
use ensogl::display::object::ObjectOps;
use ensogl_text as text;
use ensogl_theme as theme;
use wasm_bindgen::prelude::*;
use parser::Parser;



const STUB_MODULE:&str = "from Base import all\n\nmain = IO.println \"Hello\"\n";


#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_interface() {
    web::forward_panic_hook_to_console();
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



// ==================
// === Mock Types ===
// ==================

/// Allows the creation of arbitrary unique `Type`s.
#[derive(Clone,Debug,Default)]
struct DummyTypeGenerator {
    type_counter : u32
}

impl DummyTypeGenerator {
    fn get_dummy_type(&mut self) -> Type {
        self.type_counter += 1;
        Type::from(format!("dummy_type_{}",self.type_counter))
    }
}



// ========================
// === Init Application ===
// ========================

fn init(app:&Application) {

    let _bg = app.display.scene().style_sheet.var(theme::application::background);

    let world     = &app.display;
    let scene     = world.scene();
    let camera    = scene.camera();
    let navigator = Navigator::new(scene,&camera);

    app.views.register::<project::View>();
    app.views.register::<text::Area>();
    app.views.register::<GraphEditor>();
    let project_view = app.new_view::<project::View>();
    let graph_editor = project_view.graph();
    let code_editor  = project_view.code_editor();
    world.add_child(&project_view);

    code_editor.text_area().set_content(STUB_MODULE.to_owned());

    project_view.status_bar().add_event(status_bar::event::Label::new("This is a status message."));
    graph_editor.debug_push_breadcrumb();


    // === Nodes ===

    let node1_id = graph_editor.add_node();
    let node2_id = graph_editor.add_node();
    let node3_id = graph_editor.add_node();

    graph_editor.frp.set_node_position.emit((node1_id,Vector2(-150.0,50.0)));
    graph_editor.frp.set_node_position.emit((node2_id,Vector2(50.0,50.0)));
    graph_editor.frp.set_node_position.emit((node3_id,Vector2(150.0,250.0)));


    let expression_1 = expression_mock();
    graph_editor.frp.set_node_expression.emit((node1_id,expression_1.clone()));
    let expression_2 = expression_mock3();
    graph_editor.frp.set_node_expression.emit((node2_id,expression_2.clone()));

    let expression_3 = expression_mock2();
    graph_editor.frp.set_node_expression.emit((node3_id,expression_3));
    let kind       = Immutable(graph_editor::component::node::error::Kind::Panic);
    let message    = Rc::new(Some("Runtime Error".to_owned()));
    let propagated = Immutable(false);
    let error      = graph_editor::component::node::Error {kind,message,propagated};
    graph_editor.frp.set_node_error_status.emit((node3_id,Some(error)));


    // === Connections ===

    let src = graph_editor::EdgeEndpoint::new(node1_id,span_tree::Crumbs::new(default()));
    let tgt = graph_editor::EdgeEndpoint::new(node2_id,span_tree::Crumbs::new(vec![0,0,0,0,1]));
    graph_editor.frp.connect_nodes.emit((src,tgt));


    // === VCS ===

    let dummy_node_added_id     = graph_editor.add_node();
    let dummy_node_edited_id    = graph_editor.add_node();
    let dummy_node_unchanged_id = graph_editor.add_node();

    graph_editor.frp.set_node_position.emit((dummy_node_added_id,Vector2(-450.0,50.0)));
    graph_editor.frp.set_node_position.emit((dummy_node_edited_id,Vector2(-450.0,125.0)));
    graph_editor.frp.set_node_position.emit((dummy_node_unchanged_id,Vector2(-450.0,200.0)));

    let dummy_node_added_expr     = expression_mock_string("This node was added.");
    let dummy_node_edited_expr    = expression_mock_string("This node was edited.");
    let dummy_node_unchanged_expr = expression_mock_string("This node was not changed.");

    graph_editor.frp.set_node_expression.emit((dummy_node_added_id,dummy_node_added_expr));
    graph_editor.frp.set_node_expression.emit((dummy_node_edited_id,dummy_node_edited_expr));
    graph_editor.frp.set_node_expression.emit((dummy_node_unchanged_id,dummy_node_unchanged_expr));

    graph_editor.frp.set_node_vcs_status.emit((dummy_node_added_id,Some(vcs::Status::Edited)));
    graph_editor.frp.set_node_vcs_status.emit((dummy_node_edited_id,Some(vcs::Status::Added)));
    graph_editor.frp.set_node_vcs_status.emit((dummy_node_unchanged_id,Some(vcs::Status::Unchanged)));


    // === Types (Port Coloring) ===

    let mut dummy_type_generator = DummyTypeGenerator::default();

    expression_1.input_span_tree.root_ref().leaf_iter().for_each(|node|{
        if let Some(expr_id) = node.ast_id {
            let dummy_type = Some(dummy_type_generator.get_dummy_type());
            graph_editor.frp.set_expression_usage_type.emit((node1_id,expr_id,dummy_type));
        }
    });

    expression_1.output_span_tree.root_ref().leaf_iter().for_each(|node|{
        if let Some(expr_id) = node.ast_id {
            let dummy_type = Some(dummy_type_generator.get_dummy_type());
            graph_editor.frp.set_expression_usage_type.emit((node1_id,expr_id,dummy_type));
        }
    });

    expression_2.input_span_tree.root_ref().leaf_iter().for_each(|node|{
        if let Some(expr_id) = node.ast_id {
            let dummy_type = Some(dummy_type_generator.get_dummy_type());
            graph_editor.frp.set_expression_usage_type.emit((node2_id,expr_id,dummy_type));
        }
    });

    expression_2.output_span_tree.root_ref().leaf_iter().for_each(|node|{
        if let Some(expr_id) = node.ast_id {
            let dummy_type = Some(dummy_type_generator.get_dummy_type());
            graph_editor.frp.set_expression_usage_type.emit((node2_id,expr_id,dummy_type));
        }
    });

    project_view.show_prompt();

    // === Profiling ===

    let node1_status = NodeProfilingStatus::Finished { duration: 500.0 };
    graph_editor.set_node_profiling_status(node1_id, node1_status);
    let node2_status = NodeProfilingStatus::Finished { duration: 1000.0 };
    graph_editor.set_node_profiling_status(node2_id, node2_status);
    let node3_status = NodeProfilingStatus::Finished { duration: 1500.0 };
    graph_editor.set_node_profiling_status(node3_id, node3_status);


    // let tgt_type = dummy_type_generator.get_dummy_type();
    let mut was_rendered = false;
    let mut loader_hidden = false;
    let mut to_theme_switch = 100;

    world.on_frame(move |_| {
        let _keep_alive = &navigator;
        let _keep_alive = &project_view;

        if to_theme_switch == 0 {
            // println!("THEME SWITCH !!!");
            // scene.style_sheet.set("application.background",color::Rgba(0.0,0.0,0.0,1.0));
            // ensogl_theme::builtin::dark::enable(&app);
            //
            // println!(">>> {:?}", "lcha(1,0,0,1)".parse::<color::Lcha>());
        }
        to_theme_switch -= 1;

        // if i > 0 { i -= 1 } else {
        //     println!("CHANGING TYPES OF EXPRESSIONS");
        //     i = 10000;
        //     graph_editor.frp.set_node_expression.emit((node2_id,expression_2.clone()));
        //     // expression_1.input_span_tree.root_ref().leaf_iter().for_each(|node|{
        //     //     if let Some(expr_id) = node.ast_id {
        //     //         let dummy_type = Some(tgt_type.clone());
        //     //         // if j != 0 {
        //     //         //     j -= 1;
        //     //         println!("----\n");
        //     //             graph_editor.frp.set_expression_usage_type.emit((node1_id,expr_id,dummy_type));
        //     //         // } else {
        //     //         //     println!(">> null change");
        //     //             // j = 3;
        //     //             // graph_editor.frp.set_expression_usage_type.emit((node1_id,expr_id,None));
        //     //             // graph_editor.frp.set_expression_usage_type.emit((node1_id,expr_id,dummy_type));
        //     //         // };
        //     //     }
        //     // });
        // }

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

use crate::graph_editor::component::node::Expression;
use crate::graph_editor::component::node::vcs;

use ast::crumbs::*;
use ast::crumbs::PatternMatchCrumb::*;
use enso_protocol::prelude::Uuid;
use ensogl_text_msdf_sys::run_once_initialized;
use span_tree::traits::*;


pub fn expression_mock_string(label:&str) -> Expression {
    let pattern             = Some(label.to_string());
    let code                = format!("\"{}\"", label);
    let parser              = Parser::new_or_panic();
    let parameters          = vec![];
    let ast                 = parser.parse_line(&code).unwrap();
    let invocation_info     = span_tree::generate::context::CalledMethodInfo {parameters};
    let ctx                 = span_tree::generate::MockContext::new_single(ast.id.unwrap(),invocation_info);
    let output_span_tree    = span_tree::SpanTree::default();
    let input_span_tree     = span_tree::SpanTree::new(&ast,&ctx).unwrap();
    let whole_expression_id = default();
    Expression {pattern,code,whole_expression_id,input_span_tree,output_span_tree}
}

pub fn expression_mock() -> Expression {
    let pattern    = Some("var1".to_string());
    let code       = "[1,2,3]".to_string();
    let parser     = Parser::new_or_panic();
    let this_param = span_tree::ArgumentInfo {
        name : Some("this".to_owned()),
        tp   : Some("Text".to_owned()),
    };
    let parameters       = vec![this_param];
    let ast              = parser.parse_line(&code).unwrap();
    let invocation_info  = span_tree::generate::context::CalledMethodInfo {parameters};
    let ctx              = span_tree::generate::MockContext::new_single(ast.id.unwrap(),invocation_info);
    let output_span_tree = span_tree::SpanTree::default();
    let input_span_tree  = span_tree::SpanTree::new(&ast,&ctx).unwrap();
    let whole_expression_id = default();
    Expression {pattern,code,whole_expression_id,input_span_tree,output_span_tree}
}

pub fn expression_mock2() -> Expression {
    let pattern          = Some("var1".to_string());
    let pattern_cr       = vec![Seq { right: false }, Or, Or, Build];
    let val              = ast::crumbs::SegmentMatchCrumb::Body {val:pattern_cr};
    let parens_cr        = ast::crumbs::MatchCrumb::Segs {val,index:0};
    let code             = "make_maps size (distribution normal)".into();
    let output_span_tree = span_tree::SpanTree::default();
    let input_span_tree  = span_tree::builder::TreeBuilder::new(36)
        .add_child(0,14,span_tree::node::Kind::Chained,PrefixCrumb::Func)
            .add_child(0,9,span_tree::node::Kind::Operation,PrefixCrumb::Func)
                .set_ast_id(Uuid::new_v4())
                .done()
            .add_empty_child(10,span_tree::node::InsertionPointType::BeforeTarget)
            .add_child(10,4,span_tree::node::Kind::this().removable(),PrefixCrumb::Arg)
                .set_ast_id(Uuid::new_v4())
                .done()
            .add_empty_child(14,span_tree::node::InsertionPointType::Append)
            .set_ast_id(Uuid::new_v4())
            .done()
        .add_child(15,21,span_tree::node::Kind::argument().removable(),PrefixCrumb::Arg)
            .set_ast_id(Uuid::new_v4())
            .add_child(1,19,span_tree::node::Kind::argument(),parens_cr)
                .set_ast_id(Uuid::new_v4())
                .add_child(0,12,span_tree::node::Kind::Operation,PrefixCrumb::Func)
                    .set_ast_id(Uuid::new_v4())
                    .done()
                .add_empty_child(13,span_tree::node::InsertionPointType::BeforeTarget)
                .add_child(13,6,span_tree::node::Kind::this(),PrefixCrumb::Arg)
                    .set_ast_id(Uuid::new_v4())
                    .done()
                .add_empty_child(19,span_tree::node::InsertionPointType::Append)
                .done()
            .done()
        .add_empty_child(36,span_tree::node::InsertionPointType::Append)
        .build();
    let whole_expression_id = default();
    Expression {pattern,code,whole_expression_id,input_span_tree,output_span_tree}
}

pub fn expression_mock3() -> Expression {
    let pattern    = Some("Vector x y z".to_string());
    // let code       = "image.blur ((foo   bar) baz)".to_string();
    let code       = "Vector x y z".to_string();
    let parser     = Parser::new_or_panic();
    let this_param = span_tree::ArgumentInfo {
        name : Some("this".to_owned()),
        tp   : Some("Image".to_owned()),
    };
    let param0 = span_tree::ArgumentInfo {
        name : Some("radius".to_owned()),
        tp   : Some("Number".to_owned()),
    };
    let param1 = span_tree::ArgumentInfo {
        name : Some("name".to_owned()),
        tp   : Some("Text".to_owned()),
    };
    let param2 = span_tree::ArgumentInfo {
        name : Some("area".to_owned()),
        tp   : Some("Vector Int".to_owned()),
    };
    let param3 = span_tree::ArgumentInfo {
        name : Some("matrix".to_owned()),
        tp   : Some("Vector String".to_owned()),
    };
    let parameters       = vec![this_param,param0,param1,param2,param3];
    let ast              = parser.parse_line(&code).unwrap();
    let invocation_info  = span_tree::generate::context::CalledMethodInfo {parameters};
    let ctx              = span_tree::generate::MockContext::new_single(ast.id.unwrap(),invocation_info);
    let output_span_tree = span_tree::SpanTree::new(&ast,&ctx).unwrap();//span_tree::SpanTree::default();
    let input_span_tree  = span_tree::SpanTree::new(&ast,&ctx).unwrap();
    let whole_expression_id = default();
    Expression {pattern,code,whole_expression_id,input_span_tree,output_span_tree}
}
