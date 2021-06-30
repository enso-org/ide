use crate::prelude::*;

use enso_frp as frp;

use std::path::PathBuf;



// ===================
// === FolderEntry ===
// ===================

#[derive(Debug,Clone)]
pub enum FolderEntryType {
    File,
    Folder(AnyFolder),
}

#[derive(Debug,Clone)]
pub struct FolderEntry {
    pub name: String,
    pub path: PathBuf,
    pub type_: FolderEntryType,
}



// ==============
// === Folder ===
// ==============

pub trait Folder: Debug {
    fn request_entries(&self, entries_loaded:frp::Source<Vec<FolderEntry>>);
}

#[derive(Debug,Clone)]
pub struct AnyFolder(Rc<dyn Folder>);

impl<D:'static + Folder> From<D> for AnyFolder {
    fn from(dir: D) -> Self {
        AnyFolder(Rc::new(dir))
    }
}



// ===================
// === ContentRoot ===
// ===================

#[derive(Debug,Copy,Clone)]
pub enum ContentRootType {
    Project,
    Root,
    Home,
    Library,
    Custom,
}

#[derive(Debug,Clone)]
pub struct ContentRoot {
    pub name: String,
    pub path: PathBuf,
    pub type_: ContentRootType,
    pub content: AnyFolder,
}



// ==================
// === FileSystem ===
// ==================

pub trait FileSystem: Debug {
    fn request_content_roots(&self, entries_loaded:frp::Source<Vec<ContentRoot>>);
}

#[derive(Debug,Clone)]
pub struct AnyFileSystem(Rc<dyn FileSystem>);

impl<FS:'static + FileSystem> From<FS> for AnyFileSystem {
    fn from(fs: FS) -> Self {
        AnyFileSystem(Rc::new(fs))
    }
}


// === EmptyFileSystem ===

#[derive(Debug)]
struct EmptyFileSystem;

impl FileSystem for EmptyFileSystem {
    fn request_content_roots(&self, content_roots_loaded:frp::Source<Vec<ContentRoot>>) {
        content_roots_loaded.emit(vec![]);
    }
}

impl Default for AnyFileSystem {
    fn default() -> Self {
        EmptyFileSystem.into()
    }
}
