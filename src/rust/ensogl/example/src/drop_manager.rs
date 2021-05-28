use enso_prelude::*;

use ensogl_core::frp::web;
use ensogl_core::display::world::World;
use ensogl_core::display::navigation::navigator::Navigator;
use ensogl_core::display::shape::ShapeSystem;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use ensogl_core::control::io::drop_file::DropFileManager;
use ensogl_core::control::io::drop_file;

fn download_file(file:drop_file::File) {
    spawn_local(async move {
        INFO!("Received file: {file:?}");
        loop {
            match file.read_chunk().await {
                Ok(Some(chunk)) => {
                    INFO!("Received chunk: {chunk:?}");
                },
                Ok(None) => {
                    INFO!("All chunks received successfully");
                    break
                },
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
pub fn entry_point_shape_system() {
    web::forward_panic_hook_to_console();

    let world         = World::new(&web::get_html_element_by_id("root").unwrap());
    let drop_manager  = DropFileManager::new(world.scene().dom.root.as_ref());
    let network       = enso_frp::Network::new("Debug Scene");
    enso_frp::extend! { network
        let file_received = drop_manager.file_received().clone_ref();
        eval file_received ([](file) download_file(file.clone_ref()));
    }

    std::mem::forget(world);
}
