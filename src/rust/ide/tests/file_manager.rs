//! File Manager tests.

#[cfg(test)]
mod tests {
    use enso_protocol::file_manager::API;
    use enso_protocol::file_manager::*;
    use ide::*;
    use ide::transport::web::WebSocket;

    use wasm_bindgen_test::wasm_bindgen_test_configure;
    use futures::StreamExt;

    wasm_bindgen_test_configure!(run_in_browser);

    //#[wasm_bindgen_test::wasm_bindgen_test(async)]
    #[allow(dead_code)]
    async fn operations() {
        let ws        = WebSocket::new_opened("ws://localhost:30616").await;
        let ws        = ws.expect("Couldn't connect to WebSocket server.");
        let client    = Client::new(ws);
        let _executor = setup_global_executor();

        executor::global::spawn(client.runner());

        let client_id = uuid::Uuid::default();
        let session   = client.init_protocol_connection(client_id).await;
        let session   = session.expect("Couldn't initialize session.");
        let root_id   = session.content_roots[0];

        let path      = Path{root_id, segments:vec!["foo".into()]};
        let name      = "text.txt".into();
        let object    = Object{name,path};
        let object    = FileSystemObject::File(object);
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
        let read = client.read_file(move_path).await.expect("Couldn't read contents");
        assert_eq!(contents,read.contents);
    }

    //#[wasm_bindgen_test::wasm_bindgen_test(async)]
    #[allow(dead_code)]
    async fn notifications() {
        let ws         = WebSocket::new_opened("ws://localhost:30616").await;
        let ws         = ws.expect("Couldn't connect to WebSocket server.");
        let client     = Client::new(ws);
        let mut stream = client.events();

        let _executor = setup_global_executor();

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
        let object    = Object{name,path:path.clone()};
        let object    = FileSystemObject::File(object);
        client.create_file(object).await.expect("Couldn't create file.");

        let path              = Path{root_id,segments:vec!["test.txt".into()]};
        let kind              = FilesystemEventKind::Added;
        let event             = FilesystemEventInfo{path,kind};
        let file_system_event = FilesystemEvent{event};
        let notification      = Notification::FilesystemEvent(file_system_event);

        let event = stream.next().await.expect("Couldn't get any notification.");
        if let Event::Notification(incoming_notification) = event {
            assert_eq!(incoming_notification,notification);
        } else {
            panic!("Incoming event isn't a notification.");
        }
    }
}
