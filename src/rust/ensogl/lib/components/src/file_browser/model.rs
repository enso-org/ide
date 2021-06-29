use crate::prelude::*;

use std::path::PathBuf;



// ===================
// === FolderEntry ===
// ===================

#[derive(Debug,Clone)]
pub enum FolderEntryType {
    File,
    Directory(AnyFolder),
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
    fn num_entries(&self) -> usize;
    fn get_entry(&self, index:usize) -> FolderEntry;
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
    fn num_content_roots(&self) -> usize;
    fn get_content_root(&self, index:usize) -> ContentRoot;
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
    fn num_content_roots(&self) -> usize {
        0
    }

    fn get_content_root(&self, _index: usize) -> ContentRoot {
        panic!("Requesting non-existent content root")
    }
}

impl Default for AnyFileSystem {
    fn default() -> Self {
        EmptyFileSystem.into()
    }
}
