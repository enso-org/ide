//! This is a root module for all texture storage definitions.

use crate::prelude::*;



// ===============
// === Storage ===
// ===============

pub trait Storage = Debug + Default + Into<AnyStorage> + PhantomInto<AnyStorage> + 'static;

shapely::define_singleton_enum! {
    AnyStorage {RemoteImage,GpuOnly,Owned}
}

pub trait StorageRelation<InternalFormat,ElemType>: Storage {
    type Storage: Debug;
}

pub type StorageOf<S,I,T> = <S as StorageRelation<I,T>>::Storage;
