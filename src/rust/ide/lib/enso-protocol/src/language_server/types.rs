//! This module contains language server types.

use super::*;



// =============
// === Event ===
// =============

/// Event emitted by the Language Server `Client`.
pub type Event = json_rpc::handler::Event<Notification>;



// ============
// === Path ===
// ============

/// A path is a representation of a path relative to a specified content root.
// FIXME [mwu] Consider rename to something like `FilePath`, see https://github.com/luna/enso/issues/708
#[derive(Clone,Debug,Serialize,Deserialize,Hash,PartialEq,Eq)]
#[serde(rename_all="camelCase")]
pub struct Path {
    /// Path's root id.
    pub root_id:Uuid,
    /// Path's segments.
    pub segments:Vec<String>,

}

impl From<&FileSystemObject> for Path {
    fn from(file_system_object:&FileSystemObject) -> Path {
        match file_system_object {
            FileSystemObject::Directory{name,path}          => path.append_im(name),
            FileSystemObject::File{name,path}               => path.append_im(name),
            FileSystemObject::DirectoryTruncated{name,path} => path.append_im(name),
            FileSystemObject::Other{name,path}              => path.append_im(name),
            FileSystemObject::SymlinkLoop{name,path,..}     => path.append_im(name)
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "//{}/", self.root_id)?;
        write!(f, "{}", self.segments.join("/"))
    }
}

impl Path {
    /// Splits path into name and path to parent directory. e.g.:
    /// Path{root_id,segments:["foo","bar","qux"]} => ("qux",Path{root_id,segments:["foo","bar"]})
    pub fn split(mut self) -> Option<(Path,String)> {
        self.segments.pop().map(|name| (self,name))
    }

    /// Creates a new clone appending a new `segment`.
    pub fn append_im(&self, segment:impl Str) -> Self {
        let mut clone = self.clone();
        clone.segments.push(segment.into());
        clone
    }

    /// Returns the parent `Path` if the current `Path` is not `root`.
    pub fn parent(&self) -> Option<Self> {
        let mut parent = self.clone();
        parent.segments.pop().map(|_| parent)
    }

    /// Returns the file name, i.e. the last segment if exists.
    pub fn file_name(&self) -> Option<&String> {
        self.segments.last()
    }

    /// Returns the file extension, i.e. the part of last path segment after the last dot.
    /// Returns `None` is there is no segments or no dot in the last segment.
    pub fn extension(&self) -> Option<&str> {
        let name           = self.file_name()?;
        let last_dot_index = name.rfind('.')?;
        Some(&name[last_dot_index + 1..])
    }

    /// Returns the stem of filename, i.e. part of last segment without extension if present.
    pub fn file_stem(&self) -> Option<&str> {
        let name        = self.file_name()?;
        let name_length = name.rfind('.').unwrap_or_else(|| name.len());
        Some(&name[..name_length])
    }

    /// Constructs a new path from given root ID and segments.
    pub fn new(root_id:Uuid, segments:impl IntoIterator<Item:AsRef<str>>) -> Path {
        Path {
            root_id,
            segments : segments.into_iter().map(|s| s.as_ref().into()).collect()
        }
    }
}



// ====================
// === Notification ===
// ====================

/// Notification generated by the Language Server.
#[derive(Clone,Debug,PartialEq)]
#[derive(Serialize,Deserialize)]
#[serde(tag="method", content="params")]
pub enum Notification {
    /// Filesystem event occurred for a watched path.
    #[serde(rename = "file/event")]
    FileEvent(FileEvent),

    /// Sent from the server to the client to inform about new information for certain expressions
    /// becoming available.
    #[serde(rename = "executionContext/expressionValuesComputed")]
    ExpressionValuesComputed(ExpressionValuesComputed),

    /// Sent from the server to the client to inform about a failure during execution of an
    /// execution context.
    #[serde(rename = "executionContext/executionFailed")]
    ExecutionFailed(ExecutionFailed),

    /// Sent from server to the client to inform abouth the change in the suggestions database.
    #[serde(rename = "search/suggestionsDatabaseUpdates")]
    SuggestionDatabaseUpdates(SuggestionDatabaseUpdatesEvent),
}

/// Sent from the server to the client to inform about new information for certain expressions
/// becoming available.
#[derive(Clone,Debug,PartialEq)]
#[derive(Serialize,Deserialize)]
#[allow(missing_docs)]
#[serde(rename_all="camelCase")]
pub struct ExpressionValuesComputed {
    pub context_id : ContextId,
    pub updates    : Vec<ExpressionValueUpdate>,
}

/// Sent from the server to the client to inform about a failure during execution of an execution
/// context.
#[derive(Clone,Debug,PartialEq)]
#[derive(Serialize,Deserialize)]
#[allow(missing_docs)]
#[serde(rename_all="camelCase")]
pub struct ExecutionFailed {
    pub context_id : ContextId,
    pub message    : String,
}

/// The updates from `executionContext/expressionValuesComputed`.
#[derive(Clone,Debug,PartialEq)]
#[derive(Serialize,Deserialize)]
#[allow(missing_docs)]
#[serde(rename_all="camelCase")]
pub struct ExpressionValueUpdate {
    pub id          : ExpressionId,
    #[serde(rename = "type")] // To avoid collision with the `type` keyword.
    pub typename    : Option<String>,
    pub short_value : Option<String>,
    pub method_call : Option<MethodPointer>,
}


// =================
// === FileEvent ===
// =================

/// The `file/event` notification parameters.
#[derive(Clone,Debug,PartialEq)]
#[derive(Serialize,Deserialize)]
#[allow(missing_docs)]
pub struct FileEvent {
    pub path : Path,
    pub kind : FileEventKind,
}

/// Describes kind of filesystem event (was the file created or deleted, etc.)
#[derive(Clone,Copy,Debug,PartialEq)]
#[derive(Serialize,Deserialize)]
#[allow(missing_docs)]
pub enum FileEventKind {
    Added,
    Removed,
    Modified,
}



// ======================
// === FileAttributes ===
// ======================

/// Attributes of the file in the filesystem.
#[derive(Clone,Debug,PartialEq,Eq,Hash)]
#[derive(Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct FileAttributes {
    /// When the file was created.
    pub creation_time:UTCDateTime,
    /// When the file was last accessed.
    pub last_access_time:UTCDateTime,
    /// When the file was last modified.
    pub last_modified_time:UTCDateTime,
    /// What kind of file is this.
    pub kind:FileSystemObject,
    /// Size of the file in bytes.
    /// (size of files not being `RegularFile`s is unspecified).
    pub byte_size:u64,
}

/// A representation of what kind of type a filesystem object can be.
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(tag = "type")]
#[allow(missing_docs)]
pub enum FileSystemObject {
    Directory {
        name:String,
        path:Path,
    },
    /// A directory which contents have been truncated, i.e. with its subtree not listed
    /// any further due to depth limit being reached.
    // FIXME: To be clarified in https://github.com/luna/enso/issues/708
    DirectoryTruncated {
        name:String,
        path:Path,
    },
    File {
        name:String,
        path:Path,
    },
    /// Represents other, potenatially unrecognized object. Example is a broken symbolic link.
    // FIXME: To be clarified in https://github.com/luna/enso/issues/708
    Other {
        name:String,
        path:Path,
    },
    /// Represents a symbolic link that creates a loop.
    SymlinkLoop {
        name:String,
        path:Path,
        /// A target of the symlink. Since it is a loop, target is a subpath of the symlink.
        target: Path,
    }
}

impl FileSystemObject {
    /// Creates a new Directory variant.
    pub fn new_directory(path:Path) -> Option<Self> {
        path.split().map(|(path,name)| Self::Directory{name,path})
    }

    /// Creates a new DirectoryTruncated variant.
    pub fn new_directory_truncated(path:Path) -> Option<Self> {
        path.split().map(|(path,name)| Self::DirectoryTruncated{name,path})
    }

    /// Creates a new File variant.
    pub fn new_file(path:Path) -> Option<Self> {
        path.split().map(|(path,name)| Self::File{name,path})
    }

    /// Creates a new Other variant.
    pub fn new_other(path:Path) -> Option<Self> {
        path.split().map(|(path,name)| Self::Other{name,path})
    }

    /// Creates a new SymlinkLoop variant.
    pub fn new_symlink_loop(path:Path,target:Path) -> Option<Self> {
        path.split().map(|(path,name)| Self::SymlinkLoop{name,path,target})
    }
}




// ================
// === Position ===
// ================

/// A representation of a position in a text file.
#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct Position {
    pub line      : usize,
    pub character : usize
}

impls!{ From + &From <data::text::TextLocation> for Position { |location|
    Position {
        line      : location.line,
        character : location.column,
    }
}}

impls!{ Into + &Into <data::text::TextLocation> for Position { |this|
    data::text::TextLocation {
        line   : this.line,
        column : this.character,
    }
}}



// =================
// === TextRange ===
// =================

/// A representation of a range of text in a text file.
#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct TextRange {
    pub start: Position,
    pub end: Position
}

impls!{ From + &From <Range<data::text::TextLocation>> for TextRange { |range|
    TextRange {
        start : range.start.into(),
        end   : range.end.into(),
    }
}}

impls!{ Into + &Into <Range<data::text::TextLocation>> for TextRange { |this|
    this.start.into()..this.end.into()
}}



// ================
// === TextEdit ===
// ================

/// A representation of a change to a text file at a given position.
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
#[allow(missing_docs)]
pub struct TextEdit {
    pub range: TextRange,
    pub text: String
}


// ================
// === FileEdit ===
// ================

/// A versioned representation of batch edits to a file.
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
#[allow(missing_docs)]
pub struct FileEdit {
    pub path: Path,
    pub edits: Vec<TextEdit>,
    pub old_version: Sha3_224,
    pub new_version: Sha3_224
}


// ========================
// === ExecutionContext ===
// ========================

/// Execution context ID.
pub type ContextId = Uuid;

/// Execution context expression ID.
pub type ExpressionId = Uuid;

/// A configuration object for properties of the visualisation.
#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
#[serde(rename_all="camelCase")]
#[allow(missing_docs)]
pub struct VisualisationConfiguration {
    #[allow(missing_docs)]
    pub execution_context_id: ContextId,
    /// A qualified name of the module containing the expression which creates visualisation.
    pub visualisation_module: String,
    /// An enso lambda that will transform the data into expected format, i.e. `a -> a.json`.
    pub expression: String,
}

/// Used to enter deeper in the execution context stack. In general, all consequent stack items
/// should be `LocalCall`s.
#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
#[allow(missing_docs)]
pub struct LocalCall {
    pub expression_id:ExpressionId
}

/// Points to a method definition.
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
#[allow(missing_docs)]
pub struct MethodPointer {
    pub file            : Path,
    pub defined_on_type : String,
    pub name            : String
}

/// Used for entering a method. The first item on the execution context stack should always be
/// an `ExplicitCall`.
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
#[allow(missing_docs)]
pub struct ExplicitCall {
    pub method_pointer: MethodPointer,
    pub this_argument_expression: Option<String>,
    pub positional_arguments_expressions: Vec<String>
}

/// A representation of an executable position in code, used by the context execution methods.
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(missing_docs)]
pub enum StackItem {
    ExplicitCall(ExplicitCall),
    LocalCall(LocalCall)
}


// ==============================
// === CapabilityRegistration ===
// ==============================

/// `CapabilityRegistration` is used to keep track of permissions granting.
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct CapabilityRegistration {
    /// Method is the name of the capability listed in
    /// https://github.com/luna/enso/blob/main/docs/language-server/protocol-language-server.md#capabilities
    pub method: String,
    /// One of the enumerated `RegisterOptions` depending of `method`.
    pub register_options: RegisterOptions
}

impl CapabilityRegistration {
    /// Create "file/receivesTreeUpdates" capability for path.
    pub fn create_receives_tree_updates(path:Path) -> Self {
        let method           = "file/receivesTreeUpdates".to_string();
        let register_options = RegisterOptions::Path {path};
        CapabilityRegistration {method,register_options}
    }

    /// Create "text/canEdit" capability for path.
    pub fn create_can_edit_text_file(path:Path) -> Self {
        let method           = "text/canEdit".to_string();
        let register_options = RegisterOptions::Path {path};
        CapabilityRegistration {method,register_options}
    }

    /// Create "executionContext/canModify" capability for path.
    pub fn create_can_modify_execution_context(context_id:Uuid) -> Self {
        let method = "executionContext/canModify".to_string();
        let register_options = RegisterOptions::ExecutionContextId {context_id};
        CapabilityRegistration {method,register_options}
    }

    /// Create "executionContext/receivesUpdates" capability for path.
    pub fn create_receives_execution_context_updates(context_id:Uuid) -> Self {
        let method = "executionContext/receivesUpdates".to_string();
        let register_options = RegisterOptions::ExecutionContextId {context_id};
        CapabilityRegistration {method,register_options}
    }

    /// Create "search/receivesSuggestionsDatabaseUpdates" capability
    pub fn create_receives_suggestions_database_updates() -> Self {
        let method           = "search/receivesSuggestionsDatabaseUpdates".to_string();
        let register_options = RegisterOptions::None {};
        CapabilityRegistration {method,register_options}
    }
}


// =======================
// === RegisterOptions ===
// =======================

/// `capability/acquire` takes method and options specific to the method. This type represents the
/// options. The used variant must match the method. See for details:
/// https://github.com/luna/enso/blob/main/docs/language-server/protocol-language-server.md#capabilities
//TODO[ao] we cannot have one variant for each cabability due to `untagged` attribute.
// The best solution is make CapabilityRegistration an enum and write serialization and
// deserialization by hand.
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
#[allow(missing_docs)]
pub enum RegisterOptions {
    Path {path:Path},
    #[serde(rename_all="camelCase")]
    ExecutionContextId {context_id:ContextId},
    None {},
}


// ===========================
// === Suggestion Database ===
// ===========================

/// The identifier of SuggestionEntry in SuggestionDatabase.
pub type SuggestionEntryId = usize;

/// The version of Suggestion Database.
pub type SuggestionsDatabaseVersion = usize;

/// The argument of an atom, method or function suggestion.
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct SuggestionEntryArgument {
    /// The argument name.
    pub name:String,
    /// The argument type. String 'Any' is used to specify generic types.
    pub repr_type:String,
    /// Indicates whether the argument is lazy.
    pub is_suspended:bool,
    /// Optional default value.
    pub default_value:Option<String>,
}

/// The definition scope. The start and end are chars indices.
#[derive(Hash,Debug,Copy,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
#[allow(missing_docs)]
pub struct SuggestionEntryScope {
    pub start : Position,
    pub end   : Position,
}

impls!{ From + &From <RangeInclusive<data::text::TextLocation>> for SuggestionEntryScope { |range|
    SuggestionEntryScope {
        start : range.start().into(),
        end   : range.end().into(),
    }
}}

impls!{ Into + &Into <RangeInclusive<data::text::TextLocation>> for SuggestionEntryScope { |this|
    this.start.into()..=this.end.into()
}}

/// A type of suggestion entry.
#[derive(Hash,Debug,Copy,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
#[allow(missing_docs)]
pub enum SuggestionEntryType {Atom,Method,Function,Local}

/// A Suggestion Entry.
#[derive(Hash, Debug, Clone, PartialEq, Eq,Serialize,Deserialize)]
#[allow(missing_docs)]
#[serde(tag="type")]
#[serde(rename_all="camelCase")]
pub enum SuggestionEntry {
    #[serde(rename_all="camelCase")]
    Atom {
        name          : String,
        module        : String,
        arguments     : Vec<SuggestionEntryArgument>,
        return_type   : String,
        documentation : Option<String>,
    },
    #[serde(rename_all="camelCase")]
    Method {
        name          : String,
        module        : String,
        arguments     : Vec<SuggestionEntryArgument>,
        self_type     : String,
        return_type   : String,
        documentation : Option<String>,
    },
    #[serde(rename_all="camelCase")]
    Function {
        name        : String,
        module      : String,
        arguments   : Vec<SuggestionEntryArgument>,
        return_type : String,
        scope       : SuggestionEntryScope,
    },
    #[serde(rename_all="camelCase")]
    Local {
        name        : String,
        module      : String,
        return_type : String,
        scope       : SuggestionEntryScope,
    },
}

impl SuggestionEntry {
    /// Get name of the suggested entity.
    pub fn name(&self) -> &String {
        match self {
            Self::Atom     {name,..} => name,
            Self::Function {name,..} => name,
            Self::Local    {name,..} => name,
            Self::Method   {name,..} => name,
        }
    }
}

/// The entry in the suggestions database.
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
#[allow(missing_docs)]
pub struct SuggestionsDatabaseEntry {
    pub id         : SuggestionEntryId,
    pub suggestion : SuggestionEntry,
}

/// The kind of the suggestions database update.
#[derive(Hash,Debug,Copy,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[allow(missing_docs)]
pub enum SuggestionsDatabaseUpdateKind {Add,Update,Delete}

/// The update of the suggestions database.
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[allow(missing_docs)]
#[serde(tag="type")]
pub enum SuggestionsDatabaseUpdate {
    #[serde(rename_all="camelCase")]
    Add {
        id         : SuggestionEntryId,
        suggestion : SuggestionEntry,
    },
    #[serde(rename_all="camelCase")]
    Remove {
        id : SuggestionEntryId,
    },
    #[serde(rename_all="camelCase")]
    Modify {
        id          : SuggestionEntryId,
        return_type : String,
    }
}

/// Notification about change in the suggestions database.
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
#[allow(missing_docs)]
pub struct SuggestionDatabaseUpdatesEvent {
    pub updates         : Vec<SuggestionsDatabaseUpdate>,
    pub current_version : SuggestionsDatabaseVersion,
}

/// Utilities for testing code using the LS types.
pub mod test {
    use super::*;

    use crate::language_server::ExpressionId;

    /// Generate `ExpressionValueUpdate` with update for a single expression bringing only the
    /// typename.
    pub fn value_update_with_type(id:ExpressionId, typename:impl Into<String>) -> ExpressionValueUpdate {
        ExpressionValueUpdate {
            id,
            typename    : Some(typename.into()),
            method_call : None,
            short_value : None,
        }
    }
}
