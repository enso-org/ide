//! Language Server integration tests.
//!
//! They are disabled by default, as there is no CI infrastructure to run them with Lanaguage
//! Server. To run tests manually, uncomment the `#[wasm_bindgen_test::wasm_bindgen_test(async)]`
//! attributes and use wasm-bindgen test.
//!
//! Note that running Lanugage Server is expected at `SERVER_ENDPOINT` (by default localhost:30616).
//! To run the language server manually run in the `enso` repository e.g.
//! ```
//! sbt "runner/run --server --root-id 6f7d58dd-8ee8-44cf-9ab7-9f0454033641 --path $HOME/ensotmp --rpc-port 30616"
//! ```

use ide::prelude::*;

use enso_protocol::language_server::*;
use enso_protocol::types::*;
use ide::transport::web::WebSocket;
use wasm_bindgen_test::wasm_bindgen_test_configure;

/// The endpoint at which the Language Server should be accepting WS connections.
const SERVER_ENDPOINT:&str = "ws://localhost:30616";


wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test::wasm_bindgen_test(async)]
#[allow(dead_code)]
async fn file_operations() {
    ensogl::system::web::set_stdout();

    let ws        = WebSocket::new_opened(SERVER_ENDPOINT).await;
    let ws        = ws.expect("Couldn't connect to WebSocket server.");
    let client    = Client::new(ws);
    let _executor = ide::setup_global_executor();

    executor::global::spawn(client.runner());

    let client_id = uuid::Uuid::new_v4();
    let session   = client.init_protocol_connection(client_id).await;
    let session   = session.expect("Couldn't initialize session.");
    let root_id   = session.content_roots[0];

    let execution_context    = client.create_execution_context().await;
    let execution_context    = execution_context.expect("Couldn't create execution context.");
    let execution_context_id = match execution_context.can_modify.register_options {
        RegisterOptions::ExecutionContextId{context_id} => Some(context_id),
        _                                               => None
    }.expect("Couldn't get context ID.");

    // TODO[dg]:Implement integration tests for `executionContext/*` methods when these questions
    // are clarified:
    // 1. Where can we get the expression_id from?
    // 2.
    // let expression_id = uuid::Uuid::new_v4();
    // let local_call    = LocalCall {expression_id};
    // let stack_item    = StackItem::LocalCall(local_call);
    // let response      = client.push_execution_context(execution_context_id,stack_item).await;
    // response.expect("Couldn't push execution context.");
    //
    // let response = client.pop_execution_context(execution_context_id).await;
    // response.expect("Couldn't pop execution context.");

    // let visualisation_id     = uuid::Uuid::new_v4();
    // let expression_id        = uuid::Uuid::new_v4();
    // let expression           = "1 + 1".to_string();
    // let visualisation_module = "[Foo.Bar.Baz]".to_string();
    // let visualisation_config = VisualisationConfiguration
    //     {execution_context_id,expression,visualisation_module};
    // let response = client.attach_visualisation(visualisation_id,expression_id,visualisation_config);
    // response.await.expect("Couldn't attach visualisation.");
    //
    // let expression           = "1 + 1".to_string();
    // let visualisation_module = "[Foo.Bar.Baz]".to_string();
    // let visualisation_config = VisualisationConfiguration
    // {execution_context_id,expression,visualisation_module};
    // let response = client.modify_visualisation(visualisation_id,visualisation_config).await;
    // response.expect("Couldn't modify visualisation.");
    //
    // let response = client.detach_visualisation(execution_context_id,visualisation_id).await;
    // response.expect("Couldn't detach visualisation.");

    let response = client.destroy_execution_context(execution_context_id).await;
    response.expect("Couldn't destroy execution context.");

    let path      = Path{root_id, segments:vec!["foo".into()]};
    let name      = "text.txt".into();
    let object    = FileSystemObject::File {name,path};
    client.create_file(object.clone()).await.expect("Couldn't create file.");

    let file_path = Path{root_id, segments:vec!["foo".into(),"text.txt".into()]};
    let contents  = "Hello world!".to_string();
    let result    = client.write_file(file_path.clone(),contents.clone()).await;
    result.expect("Couldn't write file.");

    let response = client.file_info(file_path.clone()).await.expect("Couldn't get status.");
    assert_eq!(response.attributes.byte_size,12);
    assert_eq!(response.attributes.kind,object);

    let response = client.file_list(Path{root_id,segments:vec!["foo".into()]}).await;
    let response = response.expect("Couldn't get file list");
    assert!(response.paths.iter().any(|file_system_object| object == *file_system_object));

    let read = client.read_file(file_path.clone()).await.expect("Couldn't read contents.");
    assert_eq!(contents,read.contents);

    let new_path = Path{root_id,segments:vec!["foo".into(),"new_text.txt".into()]};
    client.copy_file(file_path,new_path.clone()).await.expect("Couldn't copy file");
    let read = client.read_file(new_path.clone()).await.expect("Couldn't read contents.");
    assert_eq!(contents,read.contents);

    let move_path = Path{root_id,segments:vec!["foo".into(),"moved_text.txt".into()]};
    let file      = client.file_exists(move_path.clone()).await;
    let file      = file.expect("Couldn't check if file exists.");
    if file.exists {
        client.delete_file(move_path.clone()).await.expect("Couldn't delete file");
        let file = client.file_exists(move_path.clone()).await;
        let file = file.expect("Couldn't check if file exists.");
        assert_eq!(file.exists,false);
    }

    client.move_file(new_path,move_path.clone()).await.expect("Couldn't move file");
    let read = client.read_file(move_path.clone()).await.expect("Couldn't read contents");
    assert_eq!(contents,read.contents);

    let receives_tree_updates   = ReceivesTreeUpdates{path:move_path.clone()};
    let register_options        = RegisterOptions::ReceivesTreeUpdates(receives_tree_updates);
    let method                  = "text/canEdit".to_string();
    let capability_registration = CapabilityRegistration {method,register_options};
    let response = client.open_text_file(move_path.clone()).await;
    let response = response.expect("Couldn't open text file.");
    assert_eq!(response.content, "Hello world!");
    assert_eq!(response.write_capability, Some(capability_registration));

    let start       = Position{line:0,character:5};
    let end         = Position{line:0,character:5};
    let range       = TextRange{start,end};
    let text        = ",".to_string();
    let text_edit   = TextEdit{range,text};
    let edits       = vec![text_edit];
    let old_version = Sha3_224::new(b"Hello world!");
    let new_version = Sha3_224::new(b"Hello, world!");
    let path        = move_path.clone();
    let edit        = FileEdit {path,edits,old_version,new_version:new_version.clone()};
    client.apply_text_file_edit(edit).await.expect("Couldn't apply edit.");

    let future = client.save_text_file(move_path.clone(),new_version).await;
    future.expect("Couldn't save file.");

    client.close_text_file(move_path.clone()).await.expect("Couldn't close text file.");

    let read = client.read_file(move_path.clone()).await.expect("Couldn't read contents.");
    assert_eq!("Hello, world!".to_string(),read.contents);
}

//#[wasm_bindgen_test::wasm_bindgen_test(async)]
#[allow(dead_code)]
async fn file_events() {
    ensogl::system::web::set_stdout();
    let ws         = WebSocket::new_opened(SERVER_ENDPOINT).await;
    let ws         = ws.expect("Couldn't connect to WebSocket server.");
    let client     = Client::new(ws);
    let mut stream = client.events();

    let _executor = ide::setup_global_executor();

    executor::global::spawn(client.runner());

    let client_id = uuid::Uuid::default();
    let session   = client.init_protocol_connection(client_id).await;
    let session   = session.expect("Couldn't initialize session.");
    let root_id   = session.content_roots[0];

    let path      = Path{root_id,segments:vec!["test.txt".into()]};
    let file      = client.file_exists(path.clone()).await;
    let file      = file.expect("Couldn't check if file exists.");
    if file.exists {
        client.delete_file(path.clone()).await.expect("Couldn't delete file");
        let file = client.file_exists(path.clone()).await;
        let file = file.expect("Couldn't check if file exists.");
        assert_eq!(file.exists,false);
    }

    let path       = Path{root_id, segments:vec![]};
    let receives_tree_updates = ReceivesTreeUpdates{path};
    let options    = RegisterOptions::ReceivesTreeUpdates(receives_tree_updates);
    let capability = client.acquire_capability("receivesTreeUpdates".into(),options).await;
    capability.expect("Couldn't acquire receivesTreeUpdates capability.");

    let path      = Path{root_id, segments:vec![]};
    let name      = "test.txt".into();
    let object    = FileSystemObject::File {name,path:path.clone()};
    client.create_file(object).await.expect("Couldn't create file.");

    let path         = Path{root_id,segments:vec!["test.txt".into()]};
    let kind         = FileEventKind::Added;
    let event        = FileEvent {path,kind};
    let notification = Notification::FileEvent {event};

    let event = stream.next().await.expect("Couldn't get any notification.");
    if let Event::Notification(incoming_notification) = event {
        assert_eq!(incoming_notification,notification);
    } else {
        panic!("Incoming event isn't a notification.");
    }
}
