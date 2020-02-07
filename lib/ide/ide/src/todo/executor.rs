//! TODO [mwu] This module is still provisional work in progress. Currently
//!            provided only for tests purposes, needs to be completed and
//!            properly reviewed.
//!
//! Module providing the executor-related types and functions.

#![allow(missing_docs)] // TODO [mwu] remove once module is done

use crate::prelude::*;

use futures::task::LocalSpawnExt;
use futures::task::LocalSpawn;
use futures::task::LocalFutureObj;
use futures::task::SpawnError;
use futures::executor::LocalPool;
use futures::executor::LocalSpawner;

use basegl::control::callback::CallbackHandle;
use basegl::control::EventLoop;

// TODO [mwu] If anything, likely thread local variable should be preferred.
static mut CURRENT_SPAWNER: Option<Box<dyn LocalSpawn>> = None;

// TODO [mwu] consider whether it is the "current" spawner or rather
//            "the global" spawner. Is it available from outside the async
//             context? Do we allow using more than one executor in the IDE?
#[allow(unsafe_code)]
pub fn set_global_spawner(spawner: impl LocalSpawn + 'static) {
    unsafe {
        CURRENT_SPAWNER = Some(Box::new(spawner));
    }
}

#[allow(unsafe_code)]
pub fn unset_global_spawner() {
    unsafe {
        CURRENT_SPAWNER = None;
    }
}

#[allow(unsafe_code)]
pub fn current_spawner() -> &'static mut Box<dyn LocalSpawn> {
    unsafe {
        CURRENT_SPAWNER.as_mut().expect("no global executor has been provided")
    }
}

/// Spawn a task scheduled within a current executor.
/// Panics, if called when there is no active asynchronous execution.
pub fn spawn_task(f:impl Future<Output=()> + 'static) {
    current_spawner().spawn_local(f).ok();
}



#[derive(Debug)]
pub struct JsExecutor {
    #[allow(dead_code)]
    executor   : Rc<RefCell<LocalPool>>,
    #[allow(dead_code)]
    event_loop : EventLoop,
    spawner    : LocalSpawner,
    #[allow(dead_code)]
    cb_handle  : CallbackHandle,
}

impl JsExecutor {
    pub fn new(event_loop:EventLoop) -> JsExecutor {
        let executor  = LocalPool::default();
        let spawner   = executor.spawner();
        let executor  = Rc::new(RefCell::new(executor));
        let cb_handle = JsExecutor::schedule_execution(event_loop.clone(),executor.clone());
        JsExecutor {executor,event_loop,spawner,cb_handle}
    }

    pub fn schedule_execution
    (event_loop:EventLoop, executor:Rc<RefCell<LocalPool>>) -> CallbackHandle {
        event_loop.add_callback(move |_| {
            // Safe, because this is the only place borrowing executor and loop
            // callback shall never be re-entrant.
            let mut executor = executor.borrow_mut();
            set_global_spawner(executor.spawner());
            executor.run_until_stalled();
            unset_global_spawner();
        })
    }

    pub fn spawn
    (&self, f:impl Future<Output = ()> + 'static)
     -> Result<(), SpawnError> {
        self.spawner.spawn_local(f)
    }

    pub fn add_callback<F:basegl::control::EventLoopCallback>
    (&mut self, callback:F) -> CallbackHandle {
        self.event_loop.add_callback(callback)
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
