//! This module implements utilities that allow for a safer usage pattern of
//! a shared data stored under `RefCell`. The primary goal is to allow accessing
//! the data without any direct calls to `borrow_mut`.
//!
//! Any type that implements `IsWeakHandle` or `IsStrongHandle` (types provided
//! out-of-the-box are Weak<RefCell<T>> and Strong<RefCell<T>> responsively)
//! gets an extension method names `with`. It allows accessing a data through
//! callback function that will get &mut access to the `T`.
//!
//! The safety is ensured by the execution model. The `with` function returns
//! a `Future` that yields a result of callback. Because `borrow_mut` only
//! applied in the `Future` implementation (being called by the executor), it
//! is guaranteed to be safe (as executors do not allow self-entry and callback
//! must be a synchronous function).

use crate::prelude::*;

use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

/// Value or error if the data under handle was not available.
pub type Result<T> = std::result::Result<T,Error>;



/// Macro defines `StrongHandle` and `WeakHandle` newtypes for handles storing
/// the type given in the argument.
///
/// This allows treating handles as separate types and fitting them with impl
/// methods of their own. Such implementation may allow
/// hiding from user gritty details of borrows usage behind nice, easy API.
#[macro_export]
macro_rules! make_handles {
    ($data_type:ty) => {
        /// newtype wrapper over StrongHandle.
        #[derive(Shrinkwrap)]
        #[derive(Clone,Debug)]
        pub struct Handle(Rc<RefCell<$data_type>>);

        impl Handle {
            /// Obtain a WeakHandle to this data.
            pub fn downgrade(&self) -> WeakHandle {
                WeakHandle(self.0.downgrade())
            }
            /// Create a new StrongHandle that will wrap given data.
            pub fn new(data:$data_type) -> StrongHandle {
                StrongHandle(Rc::new(RefCell::new(data)))
            }

            fn with_borrowed<F,R>(&self, operation:F) -> R
            where F : FnOnce(&$data_type) -> R {
                let Handle(ref ptr) = &self;
                operation(ptr.borrow_mut())
            }
        }

        /// newtype wrapper over WeakHandle.
        #[derive(Shrinkwrap)]
        #[derive(Clone,Debug)]
        pub struct WeakHandle(Weak<RefCell<$data_type>>);

        impl WeakHandle {
            /// Obtain a StrongHandle to this data.
            pub fn upgrade(&self) -> Option<Handle> {
                self.0.upgrade().map(StrongHandle)
            }
        }
    };
}

