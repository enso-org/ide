use super::prelude::*;

use crate::controller::graph::NodeTrees;
use crate::transport::test_utils::TestWithMockedTransport;
use crate::ide::IdeInitializer;

use enso_protocol::language_server::response::Completion;
use enso_protocol::project_manager;
use json_rpc::expect_call;
use json_rpc::test_util::transport::mock::MockTransport;
use serde_json::json;
use wasm_bindgen_test::wasm_bindgen_test_configure;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
async fn failure_to_open_most_recent_project_is_reported() {
    let transport   = MockTransport::new();
    let mut fixture = TestWithMockedTransport::set_up(&transport);
    fixture.run_test(async move {
        let logger           = default();
        let client           = IdeInitializer::setup_project_manager(transport);
        let name             = crate::constants::DEFAULT_PROJECT_NAME;
        let project_metadata = IdeInitializer::get_most_recent_project_or_create_new
            (&logger,&client,name).await.expect("Couldn't get most recent or create new project.");
        let client  = Rc::new(client);
        let project = IdeInitializer::open_project(&logger,client,project_metadata);
        let project = project.await;
        project.expect_err("error should have been reported");
    });
    fixture.when_stalled_send_response(json!({
            "projects": [{
                "name"       : "Project",
                "id"         : "4b871393-eef2-4970-8765-4f3c1ea83d09",
                "lastOpened" : "2020-05-08T11:04:07.28738Z"
            }]
        }));
    fixture.when_stalled_send_error(1,"Service error");
}

#[wasm_bindgen_test(async)]
async fn get_project_or_create_new() {
    let logger      = default();
    let mock_client = project_manager::MockClient::default();

    let name             = project_manager::ProjectName::new("TestProject");
    let id               = uuid::Uuid::new_v4();
    let last_opened      = default();
    let expected_project = project_manager::ProjectMetadata{name,id,last_opened};
    let projects         = vec![expected_project.clone()];
    let project_lists    = project_manager::response::ProjectList{projects};
    let count            = None;
    expect_call!(mock_client.list_projects(count) => Ok(project_lists));

    let project = IdeInitializer::get_project_or_create_new(&logger, &mock_client, "TestProject");
    let project = project.await;
    assert_eq!(expected_project, project.expect("Couldn't get project."))
}

#[wasm_bindgen_test(async)]
async fn get_most_recent_project_or_create_new() {
    let logger      = default();
    let mock_client = project_manager::MockClient::default();

    let name             = project_manager::ProjectName::new("TestProject");
    let id               = uuid::Uuid::new_v4();
    let last_opened      = default();
    let expected_project = project_manager::ProjectMetadata{name,id,last_opened};
    let projects         = vec![expected_project.clone()];
    let project_lists    = project_manager::response::ProjectList{projects};
    let count            = Some(1);
    expect_call!(mock_client.list_projects(count) => Ok(project_lists));

    let project = IdeInitializer::get_most_recent_project_or_create_new
        (&logger,&mock_client,"TestProject");
    let project = project.await;
    assert_eq!(expected_project, project.expect("Couldn't get project."))
}

#[test]
fn span_tree_args() {
    use crate::test::mock::*;

    let data    = Unified::new();
    let fixture = data.fixture_customize(|_,json_client| {
        for _ in 0..2 {
            json_client.expect.completion(|_, _, _, _, _| {
                Ok(Completion {
                    results         : vec![1],
                    current_version : default(),
                })
            });
        }
    });
    let Fixture{graph,executed_graph,searcher,suggestion_db,..} = &fixture;
    let entry = suggestion_db.lookup(1).unwrap();
    searcher.pick_completion(entry.clone_ref()).unwrap();
    let id = searcher.commit_node().unwrap();

    let get_node   = || graph.node(id).unwrap();
    let get_inputs = || NodeTrees::new(&get_node().info,executed_graph).unwrap().inputs;
    let get_param1 = || get_inputs().root_ref().leaf_iter().nth(1).and_then(|node| {
        node.parameter_info.clone()
    });

    let expected_param = model::suggestion_database::to_span_tree_param(&entry.arguments[0]);

    // TODO [mwu] The searcher inserts "Base.foo". This should work as well but needs redesigned
    //            target detection rules in the span tree.
    //


    println!("{:#?}",get_inputs().root_ref().leaf_iter().collect_vec());
    return;

    graph.set_expression(id,"Base.foo 50").unwrap();
    println!("{:#?}",get_inputs().root_ref().leaf_iter().collect_vec());


    return;
    panic!("{}",get_node().info.expression().repr());


    assert_eq!(entry.name,"foo");
    graph.set_expression(id,"foo").unwrap();
    assert_eq!(get_param1().as_ref(), Some(&expected_param));

    graph.set_expression(id,"foo Base").unwrap();
    assert_eq!(get_param1().as_ref(), Some(&expected_param));

    graph.set_expression(id,"bar").unwrap();
    assert_eq!(get_param1(), None);

    graph.set_expression(id,"bar Base").unwrap();
    assert_eq!(get_param1(), None);
}
