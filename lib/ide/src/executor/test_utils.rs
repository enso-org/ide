
use crate::prelude::*;

use futures::executor;
use crate::executor::global::set_spawner;
use crate::executor::global::spawn;



#[derive(Debug)]
pub struct TestWithLocalPoolExecutor {
    executor    : executor::LocalPool,
    is_finished : Rc<RefCell<bool>>,
}

impl TestWithLocalPoolExecutor {
    pub fn set_up() -> Self {
        let executor    = executor::LocalPool::new();
        let is_finished = Rc::new(RefCell::new(false));

        set_spawner(executor.spawner());
        Self {executor,is_finished}
    }

    pub fn run_test<Test>(&mut self, test : Test) -> &mut Self
    where Test : Future<Output=()> + 'static {
        let is_finished_clone = self.is_finished.clone_ref();
        spawn(async move {
            test.await;
            *is_finished_clone.borrow_mut() = true;
        });
        self.executor.run_until_stalled();
        self
    }

    pub fn when_stalled<Callback>(&mut self, callback:Callback) -> &mut Self
    where Callback : FnOnce() {
        if !*self.is_finished.borrow() {
            callback();
            self.executor.run_until_stalled();
        }
        self
    }
}

impl Drop for TestWithLocalPoolExecutor {
    fn drop(&mut self) {
        assert!(*self.is_finished.borrow());
    }
}
