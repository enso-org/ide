use crate::prelude::*;


// =============
// === Entry ===
// =============

#[derive(Clone,CloneRef,Debug,Default)]
pub struct Entry {
    pub label     : ImString,
    pub is_folder : Immutable<bool>,
    pub icons     : Icon,
}

#[derive(Clone,CloneRef,Debug,Default)]
pub struct Icon {
    name : ImString
}
    
// ===========
// === FRP ===
// ===========

ensogl::define_endpoints! { <ID>
    Input {
        directory_content (Vec<ID>,Entry),
        set_highlight (Vec<ID>),
    }

    Output {
        list_directory (Vec<ID>),
        highlight (Vec<ID>),
        entry_chosen (Vec<ID>),
    }
}

#[derive(Clone,CloneRef,Debug,Default)]
pub struct View<ID:Debug+Clone+'static> {
    frp : Frp<ID>,
}

impl<ID:Debug+Clone+'static> Deref for View<ID> {
    type Target = Frp<ID>;

    fn deref(&self) -> &Self::Target { &self.frp }
}
