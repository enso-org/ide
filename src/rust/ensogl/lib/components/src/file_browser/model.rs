use crate::prelude::*;

use enso_frp as frp;
use std::path::Path;
use std::path::PathBuf;



// ====================
// === FileProvider ===
// ====================

// === File ===

#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq)]
pub enum FileType {
    File,Directory
}

#[derive(Clone,Debug)]
pub struct File {
    pub name      : PathBuf,
    pub file_type : FileType,
}


// === File Provider Trait ===

pub trait FileProvider:Debug {
    fn list_files(&self, path:&Path, list_loaded:frp::Source<Vec<File>>);
}

#[derive(Clone,Debug)]
pub struct AnyFileProvider(Rc<dyn FileProvider>);

impl<D:'static + FileProvider> From<D> for AnyFileProvider {
    fn from(provider: D) -> Self {
        AnyFileProvider(Rc::new(provider))
    }
}



// ===================
// === ContentRoot ===
// ===================

#[derive(Debug,Copy,Clone,Eq,Hash,PartialEq)]
pub enum ContentRootType {
    Project,
    Root,
    Home,
    Library,
    Custom,
}

#[derive(Debug,Clone)]
pub struct ContentRoot {
    pub name    : String,
    pub path    : PathBuf,
    pub type_   : ContentRootType,
    pub content : AnyFileProvider,
}
