//! File Manager tests.

#[cfg(test)]
mod tests {
    use enso_protocol::file_manager::API;
    use enso_protocol::file_manager::*;
    use ide::*;
    use ide::transport::web::WebSocket;

    use wasm_bindgen_test::wasm_bindgen_test_configure;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test::wasm_bindgen_test(async)]
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
        client.create_file(object).await.expect("Couldn't create file.");

        let file_path = Path{root_id, segments:vec!["foo".into(),"text.txt".into()]};
        let contents  = "Hello world!".to_string();
        let result    = client.write_file(file_path.clone(),contents.clone()).await;
        result.expect("Couldn't write file.");

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
}
