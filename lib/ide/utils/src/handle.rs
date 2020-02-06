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

/// Packs given value into a Rc<RefCell<T>>.
pub fn strong<T>(t:T) -> StrongHandle<T> { Rc::new(RefCell::new(t)) }

pub type StrongHandle<T> = Rc<RefCell<T>>;

pub type WeakHandle<T> = Weak<RefCell<T>>;

#[derive(Clone,Copy,Display,Debug,Fail)]
pub enum Error {
    /// Happens when `Weak` cannot be upgraded.
    /// Likely the object (if there was an object) has been already dropped.
    HandleExpired,
    /// RefCell was already borrowed, cannot `borrow_mut`. Should never happen.
    AlreadyBorrowed,
}

/// Value or error if the data under handle was not available.
pub type Result<T> = std::result::Result<T,Error>;



// ================
// == WeakHandle ==
// ================

/// A type that can provide `Weak` handle to a value under `RefCell`.
pub trait IsWeakHandle: Clone {
    /// Type of the data stored.
    type Data;

    /// Obtain weak handle to the data.
    fn weak_handle(&self) -> WeakHandle<Self::Data>;

    #[deprecated]
    fn with_data<T>(&self, f:impl FnOnce(&mut Self::Data) -> T) -> Result<T> {
        if let Some(data_rc) = self.weak_handle().upgrade() {
            with(data_rc.try_borrow_mut(), |data|
                match data {
                    Ok(mut data) =>
                        Ok(f(&mut data)),
                    _ =>
                        Err(Error::AlreadyBorrowed),
                }
            )
        } else {
            Err(Error::HandleExpired)
        }
    }

    /// Obtain strong handle to the data.
    fn upgrade(&self) -> Option<StrongHandle<Self::Data>> {
        self.weak_handle().upgrade()
    }

    /// Returns `Future` that shall try calling `Cb` with the access to mutably
    /// borrowed data.
    fn with<Cb,R> (&self,cb:Cb) -> WeakWith<Self,Cb>
        where Cb: FnOnce(&mut Self::Data) -> R {
        WeakWith::new(self.clone(),cb)
    }
}

/// Clones the value loaded from the handle.
pub async fn try_load<Data: Clone>
(handle:&impl IsWeakHandle<Data=Data>) -> Result<Data> {
    handle.with(|data| data.clone()).await
}

/// Stores the value into the handle.
pub async fn try_store<Data>
(handle:&impl IsWeakHandle<Data=Data>, value:Data) -> Result<()> {
    handle.with(|data| *data = value).await
}

impl<T> IsWeakHandle for WeakHandle<T> {
    type Data = T;
    fn weak_handle(&self) -> WeakHandle<Self::Data> {
        self.clone()
    }
}



// ==================
// == StrongHandle ==
// ==================

pub trait IsStrongHandle: Clone {
    /// Type of the data stored.
    type Data;

    /// Obtain strong (`Rc`) handle to the data.
    fn strong_handle(&self) -> StrongHandle<Self::Data>;

    /// Obtain weak handle to the data.
    fn downgrade(&self) -> WeakHandle<Self::Data> {
        Rc::downgrade(&self.strong_handle())
    }

    /// Returns `Future` that shall call `Cb` with the access to mutably
    /// borrowed data.
    fn with<Cb,R> (&self, cb:Cb) -> StrongWith<Self,Cb>
    where Cb: FnOnce(&mut Self::Data) -> R {
        StrongWith::new(self.clone(),cb)
    }
}

/// Clones the value loaded from the handle.
pub async fn load<Data: Clone>(handle:&impl IsStrongHandle<Data=Data>) -> Data {
    handle.with(|data| data.clone()).await
}

/// Stores the value into the handle.
pub async fn store<Data>(handle:&impl IsStrongHandle<Data=Data>, value:Data) {
    handle.with(|data| *data = value).await
}

impl<T> IsStrongHandle for StrongHandle<T> {
    type Data = T;
    fn strong_handle(&self) -> StrongHandle<Self::Data> {
        self.clone()
    }
}



// ==============
// == WeakWith ==
// ==============

/// Future that shall call `cb` with the data under `handle`.
pub struct WeakWith<Handle,Cb> {
    handle : Handle,
    cb     : Option<Cb>,
}

impl<Handle,Cb> WeakWith<Handle,Cb> {
    /// Creates the value.
    pub fn new<Data,R>
    ( handle : Handle
    , cb     : Cb)
    -> WeakWith<Handle,Cb>
    where Handle : IsWeakHandle<Data = Data>,
          Cb     : FnOnce(&mut Data) -> R, {
        WeakWith {handle,cb:Some(cb)}
    }

    pin_utils::unsafe_unpinned!(cb:Option<Cb>);
}

impl <Handle,Cb,Data,R> Future for WeakWith<Handle,Cb>
    where Handle : IsWeakHandle<Data = Data>,
          Cb     : FnOnce(&mut Data) -> R, {
    type Output = std::result::Result<R,Error>;
    fn poll(mut self:Pin<&mut Self>, cx:&mut Context<'_>) -> Poll<Self::Output> {
        let result = if let Some(handle) = self.handle.upgrade() {
            let cb     = self.cb().take().unwrap();
            let result = with(handle.borrow_mut(), |mut data| cb(&mut data));
            Ok(result)
        } else {
            Err(Error::HandleExpired)?
        };
        Poll::Ready(result)
    }
}



// ================
// == StrongWith ==
// ================

/// A `Future` structure that allows processing data under the handle with
/// given callback. As the handle is owning (`Rc`) the underlying data is always
/// accessible.
pub struct StrongWith<Handle,Cb> {
    handle : Handle,
    cb     : Option<Cb>,
}

impl<Handle,Cb> StrongWith<Handle,Cb> {
    /// Create new `StrongWith` value.
    pub fn new<Data,R>
    ( handle : Handle
    , cb     : Cb)
    -> StrongWith<Handle,Cb>
    where Handle : IsStrongHandle<Data = Data>,
              Cb : FnOnce(&mut Data) -> R,
    { StrongWith {handle,cb:Some(cb)} }

    pin_utils::unsafe_unpinned!(cb:Option<Cb>);
}

impl <Handle,Cb,Data,R> Future for StrongWith<Handle,Cb>
    where Handle : IsStrongHandle<Data = Data>,
              Cb : FnOnce(&mut Data) -> R, {
    type Output = R;

    fn poll(mut self:Pin<&mut Self>, cx:&mut Context<'_>) -> Poll<Self::Output> {
        let handle = self.handle.strong_handle();
        let cb     = self.cb().take().unwrap();
        let result = with(handle.borrow_mut(), |mut data| cb(&mut data));
        Poll::Ready(result)
    }
}

mod tests {
    use super::*;

    use futures::executor::block_on;

    #[test]
    pub fn strong_handle() {
        const INITIAL_VALUE: i32 = 1000;
        let data = strong(INITIAL_VALUE);
        let data2 = data.clone();
        let fut = async move {
            let unboxed = load(&data2).await;
            assert_eq!(unboxed, INITIAL_VALUE);
            const NEW_VALUE: i32 = INITIAL_VALUE + 1;
            store(&data2, NEW_VALUE).await;
            let unboxed = load(&data2).await;
            assert_eq!(unboxed, NEW_VALUE);
        };
        block_on(fut);
    }

    #[test]
    pub fn weak_handle() {
        const INITIAL_VALUE: i32 = 1000;
        let data = strong(INITIAL_VALUE);
        let data2 = data.downgrade();
        let fut = async move {
            let unboxed = try_load(&data2).await.unwrap();
            assert_eq!(unboxed, INITIAL_VALUE);
            const NEW_VALUE: i32 = INITIAL_VALUE + 1;
            try_store(&data2, NEW_VALUE).await.unwrap();
            let unboxed = try_load(&data2).await.unwrap();
            assert_eq!(unboxed, NEW_VALUE);
        };
        block_on(fut);
    }
}