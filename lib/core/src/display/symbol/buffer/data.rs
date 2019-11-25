use crate::prelude::*;

use crate::data::seq::observable::Observable;

/// Vector with attached callbacks listening for changes.
pub type Data<T,OnSet,OnResize> = Observable<Vec<T>,OnSet,OnResize>;

/// The `Buffer` behind a shared reference with internal mutability.
pub type SharedData<T,OnSet,OnResize> = Rc<RefCell<Data<T,OnSet,OnResize>>>;
