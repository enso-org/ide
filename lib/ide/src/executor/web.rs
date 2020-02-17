//! Module defining `JsExecutor` - an executor that tries running until stalled
//! on each animation frame callback call.

use crate::prelude::*;

use futures::task::LocalSpawn;
use futures::task::LocalFutureObj;
use futures::task::SpawnError;
use futures::executor::LocalPool;
use futures::executor::LocalSpawner;

use basegl::control::callback::CallbackHandle;
use basegl::control::EventLoopCallback;
use basegl::control::EventLoop;


/// Executor. Uses a single-threaded `LocalPool` underneat, relying on basegl's
/// `EventLoop` to do as much progress as possible on every animation frame.
#[derive(Debug)]
pub struct JsExecutor {
    /// Underlying executor. Shared internally with the event loop callback.
    executor    : Rc<RefCell<LocalPool>>,
    /// Executor's spawner handle.
    pub spawner : LocalSpawner,
    /// Event loop that calls us on each frame.
    event_loop  : Option<EventLoop>,
    /// Handle to the callback - if dropped, loop would have stopped calling us.
    /// Also owns a shared handle to the `executor`.
    cb_handle   : Option<CallbackHandle>,
}

impl JsExecutor {
    /// Creates a new executor scheduled on a given event_loop.
    /// The returned executor shall keep a shared ownership over the event loop.
    pub fn new() -> JsExecutor {
        let executor  = LocalPool::default();
        let spawner   = executor.spawner();
        let executor  = Rc::new(RefCell::new(executor));
        JsExecutor {
            executor,
            spawner,
            event_loop: None,
            cb_handle: None
        }
    }

    /// Returns a callback compatible with `EventLoop` that once called shall
    /// attempt achieving as much progress on this executor's tasks as possible
    /// without stalling.
    pub fn run_callback(&self) -> impl EventLoopCallback {
        let executor = self.executor.clone();
        move |_| {
            // Safe, because this is the only place borrowing executor and loop
            // callback shall never be re-entrant.
            let mut executor = executor.borrow_mut();
            executor.run_until_stalled();
        }
    }

    /// Registers this executor to the given event's loop. From now on, event
    /// loop shall trigger this executor on each animation frame.
    pub fn run(&mut self, event_loop:EventLoop) {
        let cb = self.run_callback();

        self.cb_handle  = Some(event_loop.add_callback(cb));
        self.event_loop = Some(event_loop);
    }

    /// Stops event loop (previously assigned by `run` method) from calling this
    /// executor anymore. Does nothing if no loop was assigned.
    pub fn stop(&mut self) {
        self.cb_handle  = None;
        self.event_loop = None;
    }

    /// Creates a new running executor with its own event loop. Registers them
    /// as a global executor.
    ///
    /// Note: Caller should store or leak this `JsExecutor` so the global
    /// spawner won't be dangling.
    pub fn new_running_global() -> JsExecutor {
        let mut executor   = JsExecutor::new();
        executor.run(EventLoop::new());
        crate::executor::global::set_spawner(executor.spawner.clone());
        executor
    }
}

impl LocalSpawn for JsExecutor {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        self.spawner.spawn_local_obj(future)
    }

    fn status_local(&self) -> Result<(), SpawnError> {
        self.spawner.status_local()
    }
}
