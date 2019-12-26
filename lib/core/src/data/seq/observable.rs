use crate::prelude::*;
use crate::data::function::callback::{Callback0,Callback1};


// ==================
// === Observable ===
// ==================

/// Wrapper for array-like type. It allows attaching callbacks which fire when the underlying
/// structure changes.
#[derive(Shrinkwrap)]
#[derive(Derivative)]
#[derivative(Debug(bound="T:Debug"))]
pub struct Observable<T,OnMut,OnResize> {
    #[shrinkwrap(main_field)]
    pub data: T,
    #[derivative(Debug="ignore")]
    pub on_mut: OnMut,
    #[derivative(Debug="ignore")]
    pub on_resize: OnResize,
}

impl<T:Default,OnMut,OnResize>
Observable<T,OnMut,OnResize> {
    pub fn new(on_mut:OnMut, on_resize:OnResize) -> Self {
        let data = default();
        Self {data,on_mut,on_resize}
    }
}

impl<T:Index<Ix>,OnMut,OnResize,Ix>
Index<Ix> for Observable<T,OnMut,OnResize> {
    type Output = <T as Index<Ix>>::Output;
    #[inline]
    fn index(&self, index:Ix) -> &Self::Output {
        &self.data[index]
    }
}

impl<T:IndexMut<Ix>,OnMut:Callback1<Ix>,OnResize,Ix:Copy>
IndexMut<Ix> for Observable<T,OnMut,OnResize> {
    #[inline]
    fn index_mut(&mut self, index:Ix) -> &mut Self::Output {
        self.on_mut.call(index);
        &mut self.data[index]
    }
}

impl <T:Extend<S>,S,OnMut,OnResize:Callback0>
Extend<S> for Observable<T,OnMut,OnResize> {
    #[inline]
    fn extend<I:IntoIterator<Item=S>>(&mut self, iter:I) {
        self.on_resize.call();
        self.data.extend(iter)
    }
}
