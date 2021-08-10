//! A stub module with new searcher GUI.

use crate::prelude::*;


// =============
// === Entry ===
// =============

/// A structure describing a single entry in Searcher.
#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug,Default)]
pub struct Entry {
    pub label     : ImString,
    pub is_folder : Immutable<bool>,
    pub icon      : Icon,
}

/// The typewrapper for icon name.
#[derive(Clone,CloneRef,Debug,Default)]
pub struct Icon {
    name : ImString
}
    
// ===========
// === FRP ===
// ===========

ensogl::define_endpoints! { <ID>
    Input {
        reset(),
        directory_content (Vec<ID>,Entry),
        set_highlight (Vec<ID>),
    }

    Output {
        list_directory (Vec<ID>),
        highlight (Vec<ID>),
        entry_chosen (Vec<ID>),
    }
}

/// The Searcher View gui component.
///
/// Currently it contains only simple mechanism of requesting searcher content and printing it to
/// the console.
#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug)]
pub struct View<ID:Debug+Clone+'static> {
    pub frp : Frp<ID>,
}

impl<ID:Debug+Clone+'static> Deref for View<ID> {
    type Target = Frp<ID>;

    fn deref(&self) -> &Self::Target { &self.frp }
}

impl<ID:ToString+Debug+Clone+'static> View<ID> {
    /// Create new searcher view.
    pub fn new() -> Self {
        let logger  = Logger::new("searcher::new::View");
        let frp     = Frp::new();
        let network = &frp.network;
        enso_frp::extend!{ network
            eval frp.directory_content ([logger]((crumbs,entry)) {
                let crumbs = crumbs.iter().map(ToString::to_string).join(",");
                info!(logger,"New Searcher Entry received: [{crumbs}] -> {entry:?}");
            });

            frp.source.list_directory <+ frp.reset.constant(vec![]);
            frp.source.list_directory <+ frp.directory_content.filter_map(|(crumbs,entry)| {
                entry.is_folder.as_some(crumbs.clone())
            });
        }

        Self{frp}
    }
}

impl<ID:ToString+Debug+Clone+'static> Default for View<ID> {
    fn default() -> Self { Self::new() }
}
