use crate::prelude::*;

use crate::control::callback::CallbackMut;
use crate::control::callback::CallbackHandle;
use crate::control::callback::CallbackRegistry;
use crate::system::web;
use wasm_bindgen::prelude::Closure;
use crate::debug::stats;
use crate::debug::stats::Stats;
use crate::debug::stats::Panel;


// =================
// === EventLoop ===
// =================

// === Definition ===

/// Event loop system.
///
/// It allows registering callbacks and firing them on demand. After a callback
/// is registered, a `CallbackHandle` is returned. The callback is automatically
/// removed as soon as the handle is dropped. You can also use the `forget`
/// method on the handle to make the callback registered forever, but beware
/// that it can easily lead to memory leaks.
#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct EventLoop {
    rc: Rc<RefCell<EventLoopData>>,
}

impl EventLoop {
    /// Create and start a new event loop.
    pub fn new() -> Self {
        Self::default().init()
    }

    /// Init the event loop.
    fn init(self) -> Self {
        let data = Rc::downgrade(&self.rc);
        let main = move || { data.upgrade().map(|t| t.borrow_mut().run()); };
        with(self.rc.borrow_mut(), |mut data| {
            data.main = Some(Closure::new(main));
            data.run();
        });
        self
    }

    /// Add new callback. Returns `CallbackHandle` which when dropped, removes
    /// the callback as well.
    pub fn add_callback<F:CallbackMut>(&self, callback:F) -> CallbackHandle {
        self.rc.borrow_mut().callbacks.add(callback)
    }
}



// =====================
// === EventLoopData ===
// =====================

/// The internal state of the `EventLoop`.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct EventLoopData {
    main      : Option<Closure<dyn FnMut()>>,
    callbacks : CallbackRegistry,
    stats     : Stats,
    time      : Panel,
    fps       : Panel,
    mem       : Panel,
    main_id   : i32,
}

impl Default for EventLoopData {
    fn default() -> Self {
        let main      = default();
        let callbacks = default();
        let mut stats:Stats = Stats::new();
        let main_id = default();
        let time    = stats.add_panel(stats::FrameTimeMonitor::new());
        let fps     = stats.add_panel(stats::FpsMonitor::new());
        let mem     = stats.add_panel(stats::WasmMemoryMonitor::new());
        Self {main,callbacks,stats,time,fps,mem,main_id}
    }
}

impl EventLoopData {
    /// Create new instance.
    pub fn run(&mut self) {

        self.time.begin();
        self.fps.begin();
        self.mem.begin();
        let callbacks   = &mut self.callbacks;
        let callback_id = self.main.as_ref().map_or(default(), |main| {
            callbacks.run_all();
            web::request_animation_frame2(main)
        });
        self.main_id = callback_id;
        self.time.end();
        self.fps.end();
        self.mem.end();
        self.stats.draw();
    }
}

impl Drop for EventLoopData {
    fn drop(&mut self) {
        web::cancel_animation_frame(self.main_id).ok();
    }
}
