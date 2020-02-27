//! This module defines utilities for working with PhantomData.

use super::std_reexports::*;

/// The following `PhantomData` implementations allow each argument to be non
/// Sized. Unfortunately, this is not equivalent to `PhantomData<(T1,T2,...)>`,
/// as tuple requires each arg to implement `Sized`.
pub type PhantomData2<T1,T2>                      = PhantomData<(PhantomData <T1>,                      PhantomData<T2>)>;
pub type PhantomData3<T1,T2,T3>                   = PhantomData2<PhantomData2<T1,T2>,                   PhantomData<T3>>;
pub type PhantomData4<T1,T2,T3,T4>                = PhantomData2<PhantomData3<T1,T2,T3>,                PhantomData<T4>>;
pub type PhantomData5<T1,T2,T3,T4,T5>             = PhantomData2<PhantomData4<T1,T2,T3,T4>,             PhantomData<T5>>;
pub type PhantomData6<T1,T2,T3,T4,T5,T6>          = PhantomData2<PhantomData5<T1,T2,T3,T4,T5>,          PhantomData<T6>>;
pub type PhantomData7<T1,T2,T3,T4,T5,T6,T7>       = PhantomData2<PhantomData6<T1,T2,T3,T4,T5,T6>,       PhantomData<T7>>;
pub type PhantomData8<T1,T2,T3,T4,T5,T6,T7,T8>    = PhantomData2<PhantomData7<T1,T2,T3,T4,T5,T6,T7>,    PhantomData<T8>>;
pub type PhantomData9<T1,T2,T3,T4,T5,T6,T7,T8,T9> = PhantomData2<PhantomData8<T1,T2,T3,T4,T5,T6,T7,T8>, PhantomData<T9>>;
