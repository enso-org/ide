//! Utilities for testing executor-dependent code.

use crate::prelude::*;

use crate::executor::global::set_spawner;
use crate::executor::global::spawn;

use futures::executor;
use utils::test::traits::*;



/// A fixture for tests which makes able to run part of tests as asynchronous tasks in
/// LocalPoolExecutor. All spawned task will be run before dropping this structure - if some
/// task will be blocked, panic will be raised.
#[derive(Debug)]
pub struct TestWithLocalPoolExecutor {
    executor           : executor::LocalPool,
    running_task_count : Rc<Cell<usize>>,
}

impl TestWithLocalPoolExecutor {
    /// Set up the test fixture.
    pub fn set_up() -> Self {
        let executor           = executor::LocalPool::new();
        let running_task_count = Rc::new(Cell::new(0));

        set_spawner(executor.spawner());
        Self {executor,running_task_count}
    }

    /// Spawn new task in executor.
    pub fn run_task<Task>(&mut self, task: Task)
    where Task : Future<Output=()> + 'static {
        self.running_task_count.set(self.running_task_count.get() + 1);
        let running_tasks_clone = self.running_task_count.clone_ref();
        spawn(async move {
            task.await;
            running_tasks_clone.set(running_tasks_clone.get() - 1);
        });
    }

    /// Run all tasks until executor is stalled, and run callback then.
    ///
    /// This callback may for instance do some operations on mocks which unblocks task spawned
    /// in executor.
    pub fn when_stalled<Callback>(&mut self, callback:Callback)
    where Callback : FnOnce() {
        self.run_until_stalled();
        if self.running_task_count.get() > 0 {
            callback();
        }
    }

    /// Run all tasks until executor is stalled, and spawn a new task then.
    ///
    /// This structure is useful to ensure, that some task will be in progress before another task
    /// will be spawned, so we can test more specific asynchronous scenarios.
    pub fn when_stalled_run_task<Task>(&mut self, task : Task)
    where Task : Future<Output=()> + 'static {
        self.run_until_stalled();
        if self.running_task_count.get() > 0 {
            self.run_task(task);
        }
    }

    /// Runs all tasks in the pool and returns if no more progress can be made on any task.
    pub fn run_until_stalled(&mut self) {
        self.executor.run_until_stalled();
    }

    /// Runs all tasks until stalled. Panics, if some tasks remains then unfinished.
    pub fn expect_finished(&mut self) {
        self.run_until_stalled();
        assert_eq!(0,self.running_task_count.get(),"The tasks are not complete!");
    }

    /// Runs all tasks until stalled and tries retrieving value from the future.
    /// If the future cannot complete, panics.
    ///
    /// This function is useful when testing asynchronous code without using the `run_task` API
    /// (e.g. because we want to interleave the asynchronous task with other calls affecting its
    /// execution).
    pub fn expect_completion<R>(&mut self, fut:impl Future<Output=R>) -> R {
        self.run_until_stalled();
        fut.boxed_local().expect_ready()
    }
}

impl Drop for TestWithLocalPoolExecutor {
    fn drop(&mut self) {
        // We should be able to finish test.
        self.expect_finished();
    }
}
