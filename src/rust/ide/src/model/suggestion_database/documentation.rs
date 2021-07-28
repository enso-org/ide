//! The module contains the logic related to creating documentation from the suggestion database entries.
use crate::prelude::*;

use crate::double_representation::module::QualifiedName;
use crate::double_representation::tp;
use crate::model::suggestion_database;

use super::Entry;



// =====================
// === Documentation ===
// =====================


/// Type alias for the struct doc created for displaying the documentation for the different
/// suggestion database entries.
pub type Documentation = String;



// ===================
// === Module Docs ===
// ===================


/// Documentation for a Module. Can be converted to a `Documentation` struct for displaying in the
/// searcher.
#[derive(Debug)]
pub struct ModuleDocumentation {
    module : Rc<Entry>,
    atoms  : Vec<AtomDocs>,
    others : Vec<Rc<Entry>>,
}

impl ModuleDocumentation {
    /// Create the `Documentation` for the module. Requires the name of the module and
    /// access to a `DataStore` that contains the full documentation data.
    pub fn create_from_db(module:&QualifiedName,data:&suggestion_database::DataStore) -> Option<Self>{
        let module_entry = data.get_module(module)?;
        let module_atom_entries = data.get_module_atoms(module);
        let atom_types = module_atom_entries.iter().filter_map(|entry| {
            tp::QualifiedName::from_text(&entry.return_type).ok()
        });
        let atom_docs = atom_types.filter_map(|atom_type| {
            AtomDocs::create_from_db(&atom_type, data)
        }).collect();
        let others = data.get_module_methods(module);
        Some(ModuleDocumentation{module:module_entry,atoms:atom_docs,others})
    }
}

impl From<ModuleDocumentation> for Documentation {
    fn from(docs: ModuleDocumentation) -> Self {
        // Module documentation.
        let mut output = make_overview_doc(&docs.module);
        // Create section for atoms.
        let atom_doc:String = docs.atoms.into_iter().map_into::<Documentation>().map(|d| remove_outer_html_wrapper(&d).to_string()).collect();
        output.push_str(&wrap_in_atoms_section_container(remove_outer_html_wrapper(&atom_doc)));
        // Create section for other methods.
        let methods = create_doc_list(&docs.others);
        output.push_str(&wrap_in_methods_section_container(methods));
        // Put the whole doc in another container.
        wrap_in_doc_container(output)
    }
}

fn create_doc_list(entries: &[Rc<Entry>]) -> String {
    entries.iter().filter_map(|entry| {
        let doc         = entry.documentation_html.clone()?;
        let doc_unboxed = remove_outer_html_wrapper(&doc);
        Some(doc_unboxed.to_string())
    }).collect()
}



// =================
// === Atom Docs ===
// =================


/// Documentation for an Atom. Can be converted to a `Documentation` struct for displaying in the
/// searcher.
#[derive(Debug)]
pub struct AtomDocs {
    atom     : Rc<Entry>,
    methods : Vec<Rc<Entry>>,
}

impl AtomDocs {
    /// Create the `Documentation` for the atom. Requires the name of the atom and
    /// access to a `DataStore` that contains the full documentation data.
    pub fn create_from_db(atom_name:&tp::QualifiedName,data:&suggestion_database::DataStore) -> Option<Self>{
        let atom    = data.get_atom(atom_name)?;
        let methods = data.get_methods_for_type(atom_name);
        Some(AtomDocs{atom,methods})
    }
}

impl From<AtomDocs> for Documentation {
    fn from(docs: AtomDocs) -> Self {
        let mut output = make_overview_doc(&docs.atom);
        let method_section = create_doc_list(&docs.methods);
        output.push_str(&wrap_in_methods_section_container(method_section));
        wrap_in_doc_container(output)
    }
}



// ============================
// === Formatting Utilities ===
// ============================


const NO_DOCS_PLACEHOLDER    : &str = "<p style=\"color: #a3a6a9;\">No documentation available</p>";
const NO_ATOMS_PLACEHOLDER   : &str = "<p style=\"color: #a3a6a9;\">No atoms available</p>";
const NO_METHODS_PLACEHOLDER : &str = "<p style=\"color: #a3a6a9;\">No methods available</p>";
const METHOD_SECTION_HEADING : &str = "<p class=\"method-section-heading\">Methods</p>";
const ATOM_SECTION_HEADING   : &str = "<p class=\"atom-section-heading\">Atoms</p>";

fn make_overview_doc(entry: &Entry) -> String {
    if let Some(entry_doc) = entry.documentation_html.as_ref() {
        let unboxed_doc = remove_outer_html_wrapper(entry_doc);
        unboxed_doc.to_string()
    } else {
        format!("<div class=\"doc-title-container\"><div class=\"doc-title-name\">{}</div></div><div>{}</div>", entry.name, NO_DOCS_PLACEHOLDER)
    }
}

fn wrap_in_doc_container(s:String) -> String {
    let s = if s.is_empty() { NO_DOCS_PLACEHOLDER.to_string() } else { s };
    format!("<div class=\"enso docs\">{}</div>",s)
}

fn wrap_in_atoms_section_container(s:&str) -> String {
    let s = if s.is_empty() { NO_ATOMS_PLACEHOLDER } else { s };
    format!("<div class=\"atom-section\">{}{}</div>",ATOM_SECTION_HEADING,s)
}

fn wrap_in_methods_section_container(s:String) -> String {
    let s = if s.is_empty() { NO_METHODS_PLACEHOLDER.to_string() } else { s };
    format!("<div class=\"method-section\">{}{}</div>",METHOD_SECTION_HEADING,s)
}

/// Return the given string without prefix and postfix, but only if both can be removed. Otherwise
/// return the original string.
fn trim_prefix_postfix<'a>(s: &'a str, prefix:&str, postfix:&str) -> &'a str {
    let maybe_without_prefix = s.strip_prefix(prefix);
    let stripped = maybe_without_prefix.map(|without_prefix| without_prefix.strip_suffix(postfix));
    stripped.flatten().unwrap_or(s)
}

/// Remove the wrapper of an individual docs snippet as delivebred from the engine. This is needed
/// to be able to use the snippets in a concatenated list of items (e.g., methods).
fn remove_outer_html_wrapper(s:&str) -> &str {
    if s.is_empty() { s } else {
        let s = trim_prefix_postfix(s,"<html><body>","</body></html>");
        trim_prefix_postfix(s,"<div class=\"enso docs\">","</div>" )
    }
}
