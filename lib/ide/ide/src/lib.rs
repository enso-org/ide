#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(weak_counts)]

pub mod controller_sync;
pub mod controller_async;
pub mod todo;
pub mod web_transport;

pub mod prelude {
    pub use wasm_bindgen::prelude::*;
    pub use enso_prelude::*;
    pub use futures::{FutureExt, StreamExt};
    pub use futures::{Future, Stream};

    pub use futures::task::{Context, Poll};
    pub use core::pin::Pin;
    pub use crate::FallibleResult;
}

pub type FallibleResult<T> = std::result::Result<T,failure::Error>;

use crate::prelude::*;
use crate::web_transport::WSTransport;

use std::future::Future;
//use wasm_bindgen::JsCast;
use futures::executor::{LocalSpawner, LocalPool};
use futures::task::LocalSpawn;

use file_manager_client::Client;
use file_manager_client::Path;
//use futures::future::LocalFutureObj;
use basegl::control::callback::CallbackHandle;
use basegl::control::EventLoop;
//use js_sys::buffer;

pub struct JsExecutor {
    #[allow(dead_code)]
    executor   : Rc<RefCell<LocalPool>>,
    #[allow(dead_code)]
    event_loop : EventLoop,
    spawner    : LocalSpawner,
    #[allow(dead_code)]
    cb_handle  : basegl::control::callback::CallbackHandle,
}

impl JsExecutor {
    pub fn new(event_loop:EventLoop) -> JsExecutor {
        let executor = default::<Rc<RefCell<LocalPool>>>();
        let spawner   = executor.borrow_mut().spawner();
        let executor_ = executor.clone();
        let cb_handle = event_loop.add_callback(move |_| {
//            log!("Tick...");
            executor_.borrow_mut().run_until_stalled();
        });

        JsExecutor {executor,event_loop,spawner,cb_handle}
    }

    pub fn spawn
    (&self, f:impl Future<Output = ()> + 'static)
     -> Result<(), futures::task::SpawnError> {
        let f = Box::pin(f);
        self.spawner.spawn_local_obj(f.into())
    }

    pub fn add_callback<F:basegl::control::EventLoopCallback>
    (&mut self, callback:F) -> CallbackHandle {
        self.event_loop.add_callback(callback)
    }
}


const FILE_MANAGER_SERVER_URL: &str = "ws://localhost:9001";

pub async fn setup_file_manager(url:&str) -> Client {
    let ws = WSTransport::new(url).await;
    Client::new(ws)
}

struct IDE {
//    executor     : JsExecutor,
    pub file_manager      : Rc<RefCell<Client>>,
    pub file_manager_tick : CallbackHandle,
}

impl IDE {
    pub async fn new() -> IDE {
        log!("Creating IDE");
        let file_manager = Rc::new(RefCell::new(setup_file_manager(FILE_MANAGER_SERVER_URL).await));
        let file_manager_tick_owned_copy = file_manager.clone();
        let event_loop = leak(EventLoop::new());
        let file_manager_tick = event_loop.add_callback(move |_| {
//            log!("Ticking FM");
            file_manager_tick_owned_copy.borrow_mut().process_events();
        });

        IDE {file_manager,file_manager_tick}
    }
}



/// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub fn leak<'a,T>(t:T) ->  &'a mut T {
    Box::leak(Box::new(t))
}



//struct ModuleController {
//    data   : Rc<RefCell<Str>>,
//}
//
//impl ModuleController {
//    pub fn tick(&mut self) {
//
//    }
//
//    pub fn new() {
//        let stream_from_tetxt;
//
//
//
//        stream_from_text_ctrl.map(move |elem| {
//            self.buffer += "a";
//        });
//
////        stream_from_text_ctrl.foreach(|event| {
////            match event {
////                TextAdded() => self.…
////            }
////        })
//    }
//}

// This function is automatically invoked after the wasm module is instantiated.
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    log!("Setting up the event loop");
    let mut event_loop = EventLoop::new();
    log!("Setting up the executor");
    let mut executor: &'static mut JsExecutor = leak(JsExecutor::new(event_loop.clone()));

    log!("Spawning the first task");

    executor.spawn(async move {
        log!("Creating file manager");
        let mut ide = IDE::new().await;

        log!("File manager ready, running query");
        let path = Path::new(".");
//        let entries = ide.file_manager.borrow_mut().list(path.clone()).await;

        let entries = with(ide.file_manager.borrow_mut(), |mut fm| {
            fm.list(path.clone())
        }).await.unwrap();
        for entry in entries {
            let target_path = Path(entry.0 + "-copy");
            with(ide.file_manager.borrow_mut(), |mut fm|
                fm.copy_file(path.clone(), target_path));
        }

        let touch = with(ide.file_manager.borrow_mut(), |mut fm|
            fm.touch(Path::new("Bar.luna"))).await;
        log!("Touching Bar.lune: {:?}", touch);

        let touch2 = with(ide.file_manager.borrow_mut(), |mut fm|
            fm.touch(Path::new("Baz.luna"))).await;
        log!("Touching Baz.lune: {:?}", touch2);

        log!("Asynchronous block done");
//        log!("Exists result: {:?}", exists);
    });

    Ok(())
}
