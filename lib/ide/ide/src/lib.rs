pub mod web_transport;

pub mod prelude {
    pub use wasm_bindgen::prelude::*;
    pub use enso_prelude::*;
}

use crate::prelude::*;
use crate::web_transport::MyWebSocket;

use std::cell::RefCell;
use std::rc::Rc;
use std::future::Future;
//use futures::{FutureExt, StreamExt};
use wasm_bindgen::JsCast;
use futures::executor::{LocalSpawner, LocalPool};
use futures::task::LocalSpawn;

use file_manager_client::Client;
use file_manager_client::Path;
use futures::future::LocalFutureObj;


pub struct JsExecutorTop {
    #[allow(dead_code)]
    executor   : Rc<RefCell<LocalPool>>,
    #[allow(dead_code)]
    event_loop : basegl::control::EventLoop,
    spawner    : LocalSpawner,
    #[allow(dead_code)]
    cb_handle  : basegl::control::callback::CallbackHandle,
}

impl JsExecutorTop {
    pub fn new()-> JsExecutorTop {
        let event_loop = basegl::control::EventLoop::new();
        let executor   = default::<Rc<RefCell<LocalPool>>>();
        let spawner    = executor.borrow_mut().spawner();

        let executor_ = executor.clone();
        let cb_handle = event_loop.add_callback(move |_| {
//            log!("frame!");
            executor_.borrow_mut().run_until_stalled();
        });

        JsExecutorTop {executor,event_loop,spawner,cb_handle}
    }

    pub fn spawn
    (&self, f:impl Future<Output = ()> + 'static)
     -> Result<(), futures::task::SpawnError> {
        let f = Box::pin(f);
        self.spawner.spawn_local_obj(f.into())
    }

    pub fn add_callback<F:EventLoopCallback>
    (&self, callback:F) -> basegl::control::callback::CallbackHandle {
        self.event_loop.add_callback(callback)
    }
}








/// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn body() -> web_sys::HtmlElement {
    document().body().expect("document should have a body")
}

//fn prepend_text(line:&str) {
//    let old_text = body().text_content().unwrap_or("".into());
//    let new_text = format!("{}\n\n{}", line, old_text);
//    body().set_text_content(Some(&new_text));
//}

pub async fn setup_file_manager(url:&str) -> Client {
    let ws = MyWebSocket::new(url).await;
    Client::new(ws)
}

trait Tickable {
    fn tick(&mut self);
}

#[derive(Debug)]
pub struct EventManager {
    pub pool    : LocalPool,
    pub spawner : LocalSpawner,
}

impl EventManager {
    pub fn new() -> EventManager {
        let pool = LocalPool::new();
        let spawner = pool.spawner();
        EventManager {
            pool,
            spawner,
        }
    }

    pub fn execute<F:Future<Output=()>+'static>(&mut self, f:F) {
        let f = Box::pin(f);
        let _ = self.spawner.spawn_local_obj(f.into());
    }

    pub fn execute_cb<F,Cb>(&mut self, f:F, cb:Cb)
        where F  : Future+'static,
              Cb : FnOnce(F::Output)->()+'static {
        let f = async {
            cb(f.await);
        };
        self.execute(f);
    }
}

impl Tickable for EventManager {
    fn tick(&mut self) {
//        log!("EM tick");
        self.pool.run_until_stalled();
    }
}

// This function is automatically invoked after the wasm module is instantiated.
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut executor = Box::new(JsExecutorTop::new());
    let mut em = EventManager::new();

    let file_manager: Rc<RefCell<Option<Client>>> = Rc::new(RefCell::new(None));
    let fm2 = file_manager.clone();

    let async_sequence = async move {
        let call_exists = |path| {
            fm2.borrow_mut().as_mut().unwrap().exists(path)
        };
        let fm = setup_file_manager("ws://localhost:9001").await;
        log!("file manager created!");
        *fm2.borrow_mut() = Some(fm);

        log!("first query");
        let path = "C:/temp";
        log!("{} exists? {:?}", path, call_exists(Path::new(path)).await);

        log!("second query");
        let path = "C:/Windows";
        log!("{} exists? {:?}", path, call_exists(Path::new(path)).await);

        log!("third query");
        let path = "C:/Windows";
        log!("{} exists? {:?}", path, call_exists(Path::new(path)).await);

        log!("future done");
    };

    executor.spawner.spawn_local_obj(LocalFutureObj::from(Box::pin(async_sequence))).unwrap();

    Box::leak(executor);


    let mut i = 0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        i += 1;
//        log!("tick {}", i);
//        let text = format!("Time now: {:?}", std::time::Instant::now());

        let text = format!("requestAnimationFrame has been called {} times.", i);
        body().set_text_content(Some(&text));
        em.tick();
//        lumpen_executor(&mut fut);

        if let Ok(mut opt_fm) = file_manager.try_borrow_mut() {
//            log!("opt_fm: {:?}", opt_fm);
            if let Some(fm) = opt_fm.as_mut() {
//                log!("will tick fm");
                fm.process_events();
//                log!("buffer now: {:?};\nongoing: {:?}\nem: {:?}", fm.handler.buffer.try_borrow_mut(), fm.handler.ongoing_calls, em)
            }
        } else {
            log!("Canot tick FM, it is busy!");
        }
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}
