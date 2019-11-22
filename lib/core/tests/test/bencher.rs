use basegl::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
pub use basegl::system::web::get_performance;
pub use basegl::system::web::AnimationFrameLoop;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use super::BenchContainer;

pub struct BencherCell {
    func       : Box<dyn FnMut()>,
    container  : BenchContainer,
    iterations : usize,
    total_time : f64,
    anim_loop  : Option<AnimationFrameLoop>
}

impl BencherCell {
    pub fn new(f : Box<dyn FnMut()>, container : BenchContainer) -> Self {
        let func = f;
        let iterations = 0;
        let total_time = 0.0;
        let anim_loop = None;
        Self { func, container, iterations, total_time, anim_loop }
    }

    pub fn add_iteration_time(&mut self, time : f64) {
        self.iterations += 1;
        self.total_time += time;
        let iterations = format!("{} iterations", self.iterations);
        self.container.iter.set_inner_html(&iterations);
        let average = self.total_time / self.iterations as f64;
        let display = format!("{:.2}ms", average);
        self.container.time.set_inner_html(&display);
    }
}

#[derive(Shrinkwrap)]
pub struct BencherData {
    cell : RefCell<BencherCell>
}

impl BencherData {
    pub fn new(f : Box<dyn FnMut()>, container : BenchContainer) -> Rc<Self> {
        let cell = RefCell::new(BencherCell::new(f, container));
        Rc::new(Self { cell })
    }

    fn start(self : &Rc<Self>) {
        let data_clone = self.clone();
        let performance = get_performance().expect("Performance object");
        let mut t0 = performance.now();
        let anim_loop = AnimationFrameLoop::new(Box::new(move || {
            let mut data = data_clone.borrow_mut();
            (&mut data.func)();
            let t1 = performance.now();
            let dt = t1 - t0;
            t0 = t1;
            &data.add_iteration_time(dt);
        }));
        self.borrow_mut().anim_loop = Some(anim_loop);
    }

    fn stop(self : &Rc<Self>) {
        self.borrow_mut().anim_loop = None;
    }

    fn is_running(self : &Rc<Self>) -> bool {
        self.borrow().anim_loop.is_some()
    }
}

pub struct Bencher {
    data : Rc<BencherData>
}

impl Bencher {
    pub fn new(container : BenchContainer) -> Self {
        let func = Box::new(|| ());
        let data = BencherData::new(func, container);

        let data_clone = data.clone();
        let closure = Box::new(move || {
            if data_clone.is_running() {
                data_clone.stop();
            } else {
                data_clone.start();
            }
        }) as Box<dyn FnMut()>;
        let closure = Closure::wrap(closure);
        data.cell.borrow().container.measurement.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();

        Self { data }
    }

    pub fn iter<T, F : FnMut() -> T + 'static>(&mut self, mut func : F) {
        self.data.borrow_mut().func = Box::new(move || { func(); });
    }
}
