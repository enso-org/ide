//! An example of [`deop::Manager`] usage. The dropped files metadata and content is printed to
//! the console.

use enso_prelude::*;

use ensogl_core::display::world::World;
use ensogl_core::frp::web;
use ensogl_web::drop;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

fn download_file(file: drop::File) {
    spawn_local(async move {
        INFO!("Received file: {file:?}");
        loop {
            match file.read_chunk().await {
                Ok(Some(chunk)) => {
                    INFO!("Received chunk: {chunk:?}");
                }
                Ok(None) => {
                    INFO!("All chunks received successfully");
                    break;
                }
                Err(err) => {
                    ERROR!("Error in receiving chunk promise: {err:?}");
                    break;
                }
            }
        }
    });
}

/// The example entry point.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_drop_manager() {
    web::forward_panic_hook_to_console();

    let world = World::new(&web::get_html_element_by_id("root").unwrap());
    let drop_manager = drop::Manager::new(world.scene().dom.root.as_ref());
    let network = enso_frp::Network::new("Debug Scene");
    enso_frp::extend! { network
        let file_received = drop_manager.files_received().clone_ref();
        eval file_received ([](files) for file in files { download_file(file.clone_ref())});
    }

    let mut loader_hidden = false;
    world
        .on_frame(move |_| {
            if !loader_hidden {
                web::get_element_by_id("loader")
                    .map(|t| t.parent_node().map(|p| p.remove_child(&t).unwrap()))
                    .ok();
                loader_hidden = true;
            }
        })
        .forget();

    std::mem::forget(world);
    std::mem::forget(network);
    std::mem::forget(drop_manager);
}
