//! This module implements utilities that allow for a safer usage pattern of
//! a shared data stored under `RefCell`.



/// Macro defines `StrongHandle` and `WeakHandle` newtypes for handles storing
/// the type given in the argument.
///
/// This allows treating handles as separate types and fitting them with impl
/// methods of their own. Such implementation may allow
/// hiding from user gritty details of borrows usage behind nice, easy API.
#[macro_export]
macro_rules! make_handles {
    ($data_type:ty) => {
        /// newtype wrapper over Rc<RefCell<>>.
        #[derive(Shrinkwrap)]
        #[derive(Clone,Debug)]
        pub struct Handle(Rc<RefCell<$data_type>>);

        impl Handle {
            /// Obtain a WeakHandle to this data.
            pub fn downgrade(&self) -> WeakHandle {
                WeakHandle(self.0.downgrade())
            }
            /// Create a new StrongHandle that will wrap given data.
            pub fn new(data:$data_type) -> Self {
                Handle(Rc::new(RefCell::new(data)))
            }

            fn with_borrowed<F,R>(&self, operation:F) -> R
            where F : FnOnce(&mut $data_type) -> R {
                let Handle(ref ptr) = &self;
                operation(ptr.borrow_mut())
            }
        }

        /// newtype wrapper over Weak<RefCell<>>..
        #[derive(Shrinkwrap)]
        #[derive(Clone,Debug)]
        pub struct WeakHandle(Weak<RefCell<$data_type>>);

        impl WeakHandle {
            /// Obtain a Handle to this data.
            pub fn upgrade(&self) -> Option<Handle> {
                self.0.upgrade().map(Handle)
            }
        }
    };
}

