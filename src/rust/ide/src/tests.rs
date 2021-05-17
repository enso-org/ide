use super::prelude::*;

use crate::controller::graph::NodeTrees;
use crate::transport::test_utils::TestWithMockedTransport;
use crate::ide;

use enso_protocol::project_manager;
use enso_protocol::project_manager::{ProjectName, API, MissingComponentAction};
use json_rpc::test_util::transport::mock::MockTransport;
use serde_json::json;
use span_tree::node::InsertionPointType;
use span_tree::node;
use wasm_bindgen_test::wasm_bindgen_test_configure;
use wasm_bindgen_test::wasm_bindgen_test;
use json_rpc::Transport;

wasm_bindgen_test_configure!(run_in_browser);



// =============================================
// === JSON Rpc Transport in IDE Initializer ===
// =============================================

//#[test]
#[wasm_bindgen_test]
fn failure_to_open_project_is_reported() {
    let logger      = Logger::new("test");
    warning!(logger,"Foo");
    let mut transport   = MockTransport::new();
    let mut fixture = TestWithMockedTransport::set_up(&transport);
    {
        let logger = logger.clone();
        fixture.run_test(async move {
            // let (sender, mut receiver) = futures::channel::mpsc::unbounded();
            // transport.set_event_transmitter(sender);
            // debug!(logger,"1");
            // let next1 = receiver.next().await;
            // debug!(logger, "next1: {next1:?}");
            // let next2 = receiver.next().await;
            // debug!(logger, "next2: {next2:?}");
            //
            debug!(logger,"Starting test");
            let project_manager = Rc::new(project_manager::Client::new(transport));
            executor::global::spawn(project_manager.runner());
            debug!(logger,"1");
            let result = project_manager.list_projects(&Some(4)).await;
            debug!(logger,"result: {result:?}");
            let result = project_manager.list_projects(&Some(4)).await;
            debug!(logger,"result: {result:?}");
            //let result      = initializer.initialize_project_model().await;

            debug!(logger,"3");
            result.expect_err("Error should have been reported.");
            debug!(logger,"4");
        });
    }
    debug!(logger,"A");

    fixture.with_executor_fixture.run_until_stalled();
    debug!(logger,"A2");
    fixture.transport.mock_peer_text_message(r#"{"jsonrpc":"2.0","id":0,"result":{"projects":[{"id":"4b871393-eef2-4970-8765-4f3c1ea83d09","lastOpened":"2020-05-08T11:04:07.28738Z","name":"Unnamed"}]}}"#);

    // fixture.when_stalled_send_response(json!({
    //     "projects": [{
    //         "name"       : crate::constants::DEFAULT_PROJECT_NAME,
    //         "id"         : "4b871393-eef2-4970-8765-4f3c1ea83d09",
    //         "lastOpened" : "2020-05-08T11:04:07.28738Z"
    //     }]
    // }));
    debug!(logger,"B");
    fixture.with_executor_fixture.run_until_stalled();
    debug!(logger,"B2");
    fixture.transport.mock_peer_text_message(r#"{"jsonrpc":"2.0","id":1,"error":{"code":1,"message":"Service error","data":null}}"#);
    //fixture.when_stalled_send_error(1,"Service error");
    debug!(logger,"C");
    fixture.with_executor_fixture.run_until_stalled();
    debug!(logger,"D");
    fixture.with_executor_fixture.run_until_stalled();
    debug!(logger,"E");
}


// ====================================
// === SpanTree in Graph Controller ===
// ====================================

#[wasm_bindgen_test]
fn span_tree_args() {
    use crate::test::mock::*;
    use span_tree::Node;

    let data    = Unified::new();
    let fixture = data.fixture_customize(|_,json_client| {
        // Additional completion request happens after picking completion.
        controller::searcher::test::expect_completion(json_client,&[1]);
    });
    let Fixture{graph,executed_graph,searcher,suggestion_db,..} = &fixture;
    let entry = suggestion_db.lookup(1).unwrap();
    searcher.use_suggestion(entry.clone_ref()).unwrap();
    let id = searcher.commit_node().unwrap();

    let get_node   = || graph.node(id).unwrap();
    let get_inputs = || NodeTrees::new(&get_node().info,executed_graph).unwrap().inputs;
    let get_param  = |n| get_inputs().root_ref().leaf_iter().nth(n).and_then(|node| {
        node.argument_info()
    });
    let expected_this_param = model::suggestion_database::entry::to_span_tree_param(&entry.arguments[0]);
    let expected_arg1_param = model::suggestion_database::entry::to_span_tree_param(&entry.arguments[1]);


    // === Method notation, without prefix application ===
    assert_eq!(get_node().info.expression().repr(), "Base.foo");
    match get_inputs().root.children.as_slice() {
        // The tree here should have two nodes under root - one with given Ast and second for
        // an additional prefix application argument.
        [_,second] => {
            let Node{children,kind,..} = &second.node;
            let _expected_kind = node::Kind::insertion_point()
                .with_kind(InsertionPointType::ExpectedArgument(0));
            assert!(children.is_empty());
            // assert_eq!(kind,&node::Kind::from(expected_kind));
            assert_eq!(kind.argument_info(),Some(expected_arg1_param.clone()));
        }
        _ => panic!("Expected only two children in the span tree's root"),
    };


    // === Method notation, with prefix application ===
    graph.set_expression(id,"Base.foo 50").unwrap();
    match get_inputs().root.children.as_slice() {
        // The tree here should have two nodes under root - one with given Ast and second for
        // an additional prefix application argument.
        [_,second] => {
            let Node{children,kind,..} = &second.node;
            assert!(children.is_empty());
            // assert_eq!(kind,&node::Kind::from(node::Kind::argument()));
            assert_eq!(kind.argument_info(),Some(expected_arg1_param.clone()));
        }
        _ => panic!("Expected only two children in the span tree's root"),
    };


    // === Function notation, without prefix application ===
    assert_eq!(entry.name,"foo");
    graph.set_expression(id,"foo").unwrap();
    assert_eq!(get_param(1).as_ref(),Some(&expected_this_param));
    assert_eq!(get_param(2).as_ref(),Some(&expected_arg1_param));
    assert_eq!(get_param(3).as_ref(),None);


    // === Function notation, with prefix application ===
    graph.set_expression(id,"foo Base").unwrap();
    assert_eq!(get_param(1).as_ref(),Some(&expected_this_param));
    assert_eq!(get_param(2).as_ref(),Some(&expected_arg1_param));
    assert_eq!(get_param(3).as_ref(),None);


    // === Changed function name, should not have known parameters ===
    graph.set_expression(id,"bar").unwrap();
    assert_eq!(get_param(1),None);
    assert_eq!(get_param(2),None);
    assert_eq!(get_param(3),None);

    graph.set_expression(id,"bar Base").unwrap();
    assert_eq!(get_param(1),Some(default()));
    assert_eq!(get_param(2),Some(span_tree::ArgumentInfo::this(None)));
    assert_eq!(get_param(3),Some(default())); // FIXME: is this correct?

    graph.set_expression(id,"Base.bar").unwrap();
    assert_eq!(get_param(1),Some(span_tree::ArgumentInfo::this(None)));
    assert_eq!(get_param(2),Some(default()));
    assert_eq!(get_param(3),None);

    // === Oversaturated call ===
    graph.set_expression(id,"foo Base 10 20 30").unwrap();
    assert_eq!(get_param(1).as_ref(),Some(&expected_this_param));
    assert_eq!(get_param(2).as_ref(),Some(&expected_arg1_param));
    assert_eq!(get_param(3).as_ref(),Some(&default()));
}
