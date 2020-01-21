use crate::prelude::*;

use super::BenchContainer;
use crate::system::web::get_performance;
use basegl::control::EventLoop;
use basegl::control::callback::CallbackHandle;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use std::cell::RefCell;


// =========================
// === BencherProperties ===
// =========================

/// Cell, used to hold Bencher's data
pub struct BencherProperties {
    callback       : Box<dyn FnMut()>,
    container      : BenchContainer,
    iterations     : usize,
    total_time     : f64,
    event_loop     : EventLoop,
    callback_guard : Option<CallbackHandle>
}

impl BencherProperties {
    pub fn new<T:FnMut() + 'static>
    (event_loop:EventLoop, callback:T, container:BenchContainer) -> Self {
        let iterations     = 0;
        let total_time     = 0.0;
        let callback_guard = None;
        let callback       = Box::new(callback);
        Self {callback,container,iterations,total_time,event_loop,callback_guard}
    }

    /// Adds the duration of the next iteration and updates the UI.
    pub fn add_iteration_time(&mut self, time : f64) {
        self.iterations += 1;
        self.total_time += time;
        let iterations   = format!("{} iterations", self.iterations);
        let average      = self.total_time / self.iterations as f64;
        let display      = format!("{:.2}ms", average);

        self.container.iter.set_inner_html(&iterations);
        self.container.time.set_inner_html(&display);
    }
}



// ===================
// === BencherData ===
// ===================

#[derive(Shrinkwrap)]
pub struct BencherData {
    properties: RefCell<BencherProperties>
}

impl BencherData {
    pub fn new<T:FnMut() + 'static>
    ( event_loop:EventLoop
    , callback:T
    , container:BenchContainer) -> Rc<Self> {
        let properties = RefCell::new(BencherProperties::new(event_loop,callback,container));
        Rc::new(Self {properties})
    }

    /// Starts the benchmarking loop.
    fn start(self:&Rc<Self>) {
        let data_clone = self.clone();
        let performance = get_performance().expect("Performance object");
        let mut t0 = performance.now();
        let callback_guard = self.event_loop().add_callback(Box::new(move |_| {
            let mut data = data_clone.borrow_mut();

            (&mut data.callback)();

            let t1 = performance.now();
            let dt = t1 - t0;
            t0     = t1;

            data.add_iteration_time(dt);
        }));
        self.properties.borrow_mut().callback_guard = Some(callback_guard);
    }

    /// Stops the benchmarking loop.
    fn stop(&self) {
        self.properties.borrow_mut().callback_guard = None;
    }

    fn iter<T, F:FnMut() -> T + 'static>(&self, mut callback:F) {
        self.properties.borrow_mut().callback = Box::new(move || { callback(); });
    }

}


// === Getters ===

impl BencherData {
    fn event_loop(&self) -> EventLoop {
        self.properties.borrow().event_loop.clone()
    }

    /// Check if the loop is running.
    fn is_running(self:&Rc<Self>) -> bool {
        self.properties.borrow().callback_guard.is_some()
    }
}



// ===============
// === Bencher ===
// ===============

/// The Bencher struct with an API compatible to Rust's test Bencher.
#[derive(Clone)]
pub struct Bencher {
    data : Rc<BencherData>
}

impl Bencher {
    /// Creates a Bencher with a html test container.
    pub fn new(container:BenchContainer) -> Self {
        let func       = Box::new(|| ());
        let event_loop = EventLoop::new();
        let data       = BencherData::new(event_loop, func, container);

        let data_clone = data.clone();
        let closure = Box::new(move || {
            if data_clone.is_running() {
                data_clone.stop();
            } else {
                data_clone.start();
            }
        }) as Box<dyn FnMut()>;

        {
            let closure = Closure::wrap(closure);
            let cell = data.properties.borrow();
            let measurement = &cell.container.measurement;
            measurement.set_onclick(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }

        Self {data}
    }

    pub fn is_running(&self) -> bool {
        self.data.is_running()
    }

    /// Callback for benchmark functions to run in their body.
    pub fn iter<T, F:FnMut() -> T + 'static>(&mut self, callback:F) {
        self.data.iter(callback);
    }

    pub fn event_loop(&self) -> EventLoop {
        self.data.event_loop()
    }
}
