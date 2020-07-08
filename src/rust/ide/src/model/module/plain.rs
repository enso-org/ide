use crate::prelude::*;

use parser::api::{ParsedSourceFile, SourceFile};
use crate::model::module::{Metadata, NodeMetadata, NodeMetadataNotFound, Path};
use crate::model::module::Notification;
use crate::notification;
use flo_stream::Subscriber;
use crate::double_representation::definition::DefinitionInfo;
use crate::model::module::Content;
use data::text::{TextChange, TextLocation};
use parser::Parser;

/// A structure describing the module.
///
/// It implements internal mutability pattern, so the state may be shared between different
/// controllers. Each change in module will emit notification for each module representation
/// (text and graph).
#[derive(Debug)]
pub struct Module {
    path          : model::module::Path,
    content       : RefCell<Content>,
    notifications : notification::Publisher<Notification>,
}

impl Default for Module {
    fn default() -> Self {
        let ast = ast::known::Module::new(ast::Module{lines:default()},None);
        Self::new(ast,default())
    }
}

impl Module {
    /// Create state with given content.
    pub fn new(path:model::module::Path, ast:ast::known::Module, metadata:Metadata) -> Self {
        Module {
            path,
            content       : RefCell::new(ParsedSourceFile{ast,metadata}),
            notifications : default(),
        }
    }

    /// Create module state from given code, id_map and metadata.
    #[cfg(test)]
    pub fn from_code_or_panic<S:ToString>
    (path:model::module::Path, code:S, id_map:ast::IdMap, metadata:Metadata) -> Self {
        let parser = parser::Parser::new_or_panic();
        let ast    = parser.parse(code.to_string(),id_map).unwrap().try_into().unwrap();
        Self::new(path,ast,metadata)
    }

    fn notify(&self, notification:Notification) {
        let notify  = self.notifications.publish(notification);
        executor::global::spawn(notify);
    }
}


// === Access to Module Content ===

impl model::module::API for Module {
    fn subscribe(&self) -> Subscriber<Notification> {
        self.notifications.subscribe()
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn serialized_content(&self) -> FallibleResult<SourceFile> {
        self.content.borrow().serialize().map_err(|e| e.into())
    }

    fn ast(&self) -> ast::known::Module {
        self.content.borrow().ast.clone_ref()
    }

    fn find_definition
    (&self,id:&double_representation::graph::Id) -> FallibleResult<DefinitionInfo> {
        let ast = self.content.borrow().ast.clone_ref();
        double_representation::module::get_definition(&ast, id)
    }

    fn node_metadata(&self, id:ast::Id) -> FallibleResult<NodeMetadata> {
        let data = self.content.borrow().metadata.ide.node.get(&id).cloned();
        data.ok_or_else(|| NodeMetadataNotFound(id).into())
    }

    fn update_whole(&self, content:Content) {
        *self.content.borrow_mut() = content;
        self.notify(Notification::Invalidate);
    }

    fn update_ast(&self, ast:ast::known::Module) {
        self.content.borrow_mut().ast  = ast;
        self.notify(Notification::Invalidate);
    }

    fn apply_code_change
    (&self, change:TextChange, parser:&Parser, new_id_map:ast::IdMap) -> FallibleResult<()> {
        let mut code          = self.ast().repr();
        let replaced_location = TextLocation::convert_range(&code,&change.replaced);
        change.apply(&mut code);
        let new_ast = parser.parse(code,new_id_map)?.try_into()?;
        self.content.borrow_mut().ast = new_ast;
        self.notify(Notification::CodeChanged {change,replaced_location});
        Ok(())
    }

    fn set_node_metadata(&self, id:ast::Id, data:NodeMetadata) {
        self.content.borrow_mut().metadata.ide.node.insert(id, data);
        self.notify(Notification::MetadataChanged);
    }

    fn remove_node_metadata(&self, id:ast::Id) -> FallibleResult<NodeMetadata> {
        let lookup = self.content.borrow_mut().metadata.ide.node.remove(&id);
        let data   = lookup.ok_or_else(|| NodeMetadataNotFound(id))?;
        self.notify(Notification::MetadataChanged);
        Ok(data)
    }

    fn with_node_metadata(&self, id:ast::Id, fun:impl FnOnce(&mut NodeMetadata)) {
        let lookup   = self.content.borrow_mut().metadata.ide.node.remove(&id);
        let mut data = lookup.unwrap_or_default();
        fun(&mut data);
        self.content.borrow_mut().metadata.ide.node.insert(id, data);
        self.notify(Notification::MetadataChanged);
    }
}