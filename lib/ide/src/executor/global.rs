//! This module defines a shared, global state that stores a task spawner.
//!
//! It is expected that as a part of one-time initialization routine, IDE shall
//! call `set_spawner` to define a global spawner. Once that is done, the whole
//! codebase can use `spawn` function to schedule new tasks on the global
//! executor.
//!
//! The executor responsible for spawning tasks is expected to remain alive
//! indefinitely, at least until the last `spawn` call has been made.


use crate::prelude::*;

use futures::task::LocalSpawnExt;
use futures::task::LocalSpawn;

/// Global spawner handle.
///
/// It should be set up once, as part of the IDE initialization routine and
/// remain accessible indefinitely.
static mut SPAWNER: Option<Box<dyn LocalSpawn>> = None;

/// Sets the global spawner. It will remain accessible until it is re-set to
/// something else.
///
/// Caller should also ensure that the spawner will remain functional the whole
/// time, so e.g. it must not drop the executor connected with this spawner.
#[allow(unsafe_code)]
pub fn set_spawner(spawner_to_set:impl LocalSpawn + 'static) {
    // Note [Global Executor Safety]
    unsafe {
        SPAWNER = Some(Box::new(spawner_to_set));
    }
}

// Note [Global Executor Safety]
// =============================
// This is safe, because the global mutable state is only accessible through the
// two functions provided in this module, the code will be used only in a web
// single-threaded environment (so no race conditions are possible), and the
// functions do not allow leaking reference to the global executor.

/// Obtains the reference to the current spawner.
///
/// Panics, if the global spawner hasn't been set previously with `set_spawner`.
#[allow(unsafe_code)]
pub fn with_spawner<R>(f:impl FnOnce(&dyn LocalSpawn) -> R) -> R {
    let error_msg = "No global executor has been provided.";
    // Note: [Global Executor Safety]
    unsafe {
        let s = SPAWNER.as_mut().expect(error_msg);
        f(s)
    }
}

/// Spawns a task using the global spawner.
/// Panics, if called when there is no active asynchronous execution.
pub fn spawn(f:impl Future<Output=()> + 'static) {
    let error_msg = "Failed to spawn the task. Global executor might have been dropped.";
    with_spawner(|s| s.spawn_local(f).expect(error_msg));
}

