
use crate::prelude::*;

use enso_protocol::language_server::SuggestionId;
use enso_protocol::language_server::SuggestionsDatabaseModification;

use crate::double_representation::module::QualifiedName;
use crate::double_representation::tp;
use crate::model::module::MethodId;
use crate::model::suggestion_database::Entry;
use crate::model::suggestion_database::UpdateError;
use crate::model::suggestion_database::entry::Kind;
use crate::prelude::data::text::TextLocation;


#[derive(Debug,Default)]
pub struct DataStore {
    storage: HashMap<SuggestionId,Rc<Entry>>,
}

impl DataStore {
    pub fn from_entries(entries:impl IntoIterator<Item=(SuggestionId, Entry)>) -> DataStore {
        let mut data_store = Self::default();
        let entries = entries.into_iter().map(|(id,entry)| (id,Rc::new(entry)));
        data_store.storage.extend(entries);
        data_store
    }

    pub fn insert_entries<'a>(&mut self, entries:impl IntoIterator<Item=(&'a SuggestionId,&'a Entry)>) {
        entries.into_iter().for_each(|item| self.insert_entry(item))
    }

    pub fn insert_entry(&mut self, entry:(&SuggestionId,&Entry)) {
        self.storage.insert(*entry.0,Rc::new(entry.1.clone()));
    }

    pub fn remove_entry(&mut self, id:SuggestionId) -> Option<Rc<Entry>> {
        self.storage.remove(&id)
    }

    pub fn update_entry(&mut self, id: SuggestionId, modification:SuggestionsDatabaseModification) -> Result<(),UpdateError>{
        if let Some(old_entry) = self.storage.get_mut(&id) {
            let entry  = Rc::make_mut(old_entry);
            let errors = entry.apply_modifications(modification);
            if errors.is_empty() {
                Ok(())
            } else {
                Err(UpdateError::UpdateFailures(errors))
            }
        } else {
            Err(UpdateError::InvalidEntry(id))
        }
    }

    pub fn get_entry(&self, id: SuggestionId) -> Option<Rc<Entry>> {
        self.storage.get(&id).cloned()
    }

    pub fn get_method(&self, id:MethodId) -> Option<Rc<Entry>>{
        self.storage.values().find(|entry| entry.method_id().contains(&id)).cloned()
    }

    pub fn get_entry_by_name_and_location(&self, name:impl Str, module:&QualifiedName, location:TextLocation) -> Vec<Rc<Entry>>{
        self.storage.values().filter(|entry| {
            entry.matches_name(name.as_ref()) && entry.is_visible_at(module,location)
        }).cloned().collect()
    }

    pub fn get_locals_by_name_and_location(&self, name:impl Str, module:&QualifiedName, location:TextLocation) -> Vec<Rc<Entry>>{
        self.storage.values().filter(|entry| {
            let is_local = entry.kind == Kind::Function || entry.kind == Kind::Local;
            is_local && entry.matches_name(name.as_ref()) && entry.is_visible_at(module,location)
        }).cloned().collect()
    }

    pub fn get_module_method(&self, name:impl Str, module:&QualifiedName) ->Option<Rc<Entry>> {
        self.storage.values().find(|entry| {
            let is_method             = entry.kind == Kind::Method;
            let is_defined_for_module = entry.has_self_type(module);
            is_method && is_defined_for_module && entry.matches_name(name.as_ref())
        }).cloned()
    }

    pub fn get_module_methods(&self, module:&QualifiedName) -> Vec<Rc<Entry>> {
        self.storage.values().filter(|entry| {
            let is_method             = entry.kind == Kind::Method;
            let is_defined_for_module = entry.has_self_type(module);
            is_method && is_defined_for_module
        }).cloned().collect()
    }

    pub fn get_module_atoms(&self, module:&QualifiedName) -> Vec<Rc<Entry>> {
        self.storage.values().filter(|entry| {
            let is_atom               = entry.kind == Kind::Atom;
            let is_defined_for_module = entry.module == *module;
            is_atom && is_defined_for_module
        }).cloned().collect()
    }

    pub fn get_module(&self, module:&QualifiedName) -> Option<Rc<Entry>> {
        self.storage.values().find(|entry| {
            let is_method             = entry.kind == Kind::Module;
            let is_defined_for_module = entry.module == *module;
            is_method && is_defined_for_module
        }).cloned()
    }

    pub fn get_atom(&self, name:&tp::QualifiedName) -> Option<Rc<Entry>> {
        self.storage.values().find(|entry| {
            let is_atom     = entry.kind == Kind::Atom;
            let matches_name = entry.qualified_name() == *name;
            is_atom && matches_name
        }).cloned()
    }

    pub fn get_methods_for_type(&self, tp:&tp::QualifiedName) -> Vec<Rc<Entry>> {
        self.storage.values().filter(|entry| {
            let is_method             = entry.kind == Kind::Method;
            let is_defined_for_type   = entry.has_self_type(tp);
            is_method && is_defined_for_type
        }).cloned().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    pub fn entry_count(&self) -> usize {
        self.storage.len()
    }
}
