use crate::prelude::*;

use enso_frp as frp;

use std::path::PathBuf;



#[derive(Debug,Copy,Clone)]
pub enum FolderType {
    Standard,
    Project,
    Root,
    Home,
    Library,
    Custom,
}

#[derive(Debug,Clone)]
pub enum EntryType {
    File,
    Folder {
        type_   : FolderType,
        content : AnyFolderContent,
    },
}

#[derive(Debug,Clone)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub type_: EntryType,
}



pub trait FolderContent: Debug {
    fn request_entries(&self, entries_loaded:frp::Any<Rc<Vec<Entry>>>);
}

#[derive(Debug,Clone)]
pub struct AnyFolderContent(Rc<dyn FolderContent>);

impl Deref for AnyFolderContent {
    type Target = dyn FolderContent;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<D:'static + FolderContent> From<D> for AnyFolderContent {
    fn from(dir: D) -> Self {
        AnyFolderContent(Rc::new(dir))
    }
}


// === EmptyFolder ===

#[derive(Debug)]
struct EmptyFolderContent;

impl FolderContent for EmptyFolderContent {
    fn request_entries(&self, entries_loaded:frp::Any<Rc<Vec<Entry>>>) {
        entries_loaded.emit(Rc::new(vec![]));
    }
}

impl Default for AnyFolderContent {
    fn default() -> Self {
        EmptyFolderContent.into()
    }
}
