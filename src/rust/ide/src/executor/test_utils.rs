
use crate::prelude::*;

use futures::executor;
use crate::executor::global::set_spawner;
use crate::executor::global::spawn;



#[derive(Debug)]
pub struct TestWithLocalPoolExecutor {
    executor      : executor::LocalPool,
    running_tasks : Rc<Cell<usize>>,
}

impl TestWithLocalPoolExecutor {
    pub fn set_up() -> Self {
        let executor      = executor::LocalPool::new();
        let running_tasks = Rc::new(Cell::new(0));

        set_spawner(executor.spawner());
        Self {executor,running_tasks}
    }

    pub fn run_task<Task>(&mut self, task: Task)
    where Task : Future<Output=()> + 'static {
        self.running_tasks.set(self.running_tasks.get() + 1);
        let running_tasks_clone = self.running_tasks.clone_ref();
        spawn(async move {
            task.await;
            running_tasks_clone.set(running_tasks_clone.get() - 1);
        });
    }

    pub fn when_stalled<Callback>(&mut self, callback:Callback)
    where Callback : FnOnce() {
        self.executor.run_until_stalled();
        if self.running_tasks.get() > 0 {
            callback();
        }
    }

    pub fn when_stalled_run_task<Task>(&mut self, task : Task)
    where Task : Future<Output=()> + 'static {
        self.executor.run_until_stalled();
        if self.running_tasks.get() > 0 {
            self.run_task(task);
        }
    }
}

impl Drop for TestWithLocalPoolExecutor {
    fn drop(&mut self) {
        // We should be able to finish test.
        self.executor.run_until_stalled();
        assert_eq!(0,self.running_tasks.get());
    }
}
