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
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Path {
    /// Path's root id.
    pub root_id: Uuid,
    /// Path's segments.
    pub segments: Vec<String>,
}

impl Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "//{}/", self.root_id)?;
        write!(f, "{}", self.segments.join("/"))
    }
}


// ====================
// === Notification ===
// ====================

/// Notification generated by the Language Server.
#[derive(Clone, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum Notification {
    /// Filesystem event occurred for a watched path.
    #[serde(rename = "file/event")]
    FileEvent {
        /// The `file/event` notification input wrapper.
        /// The serialization format requires the information to be wrapped into a field named
        /// "event". This behavior is currently not specified by the specification and the issue
        /// has been raised to address this: https://github.com/luna/enso/issues/707
        // TODO [mwu] Update as the issue is resolved on way or another.
        event: FileEvent,
    }
}


// =================
// === FileEvent ===
// =================

/// The `file/event` notification parameters.
#[derive(Clone, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct FileEvent {
    pub path: Path,
    pub kind: FileEventKind,
}

/// Describes kind of filesystem event (was the file created or deleted, etc.)
#[derive(Clone, Copy, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileAttributes {
    /// When the file was created.
    pub creation_time: UTCDateTime,
    /// When the file was last accessed.
    pub last_access_time: UTCDateTime,
    /// When the file was last modified.
    pub last_modified_time: UTCDateTime,
    /// What kind of file is this.
    pub kind: FileSystemObject,
    /// Size of the file in bytes. (size of files not being `RegularFile`s is unspecified).
    pub byte_size: u64,
}

/// A representation of what kind of type a filesystem object can be.
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(missing_docs)]
pub enum FileSystemObject {
    Directory {
        name: String,
        path: Path,
    },
    /// A directory which contents have been truncated, i.e. with its subtree not listed
    /// any further due to depth limit being reached.
    // FIXME: To be clarified in https://github.com/luna/enso/issues/708
    DirectoryTruncated {
        name: String,
        path: Path,
    },
    File {
        name: String,
        path: Path,
    },
    /// Represents other, potenatially unrecognized object. Example is a broken symbolic link.
    // FIXME: To be clarified in https://github.com/luna/enso/issues/708
    Other {
        name: String,
        path: Path,
    },
    /// Represents a symbolic link that creates a loop.
    SymlinkLoop {
        name: String,
        path: Path,
        /// A target of the symlink. Since it is a loop, target is a subpath of the symlink.
        target: Path,
    }
}


// ================
// === Position ===
// ================

/// A representation of a position in a text file.
#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct Position {
    pub line: u32,
    pub character: u32
}


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


// ================
// === TextEdit ===
// ================

/// A representation of a change to a text file at a given position.
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct VisualisationConfiguration {
    #[allow(missing_docs)]
    pub execution_context_id: Uuid,
    /// A qualified name of the module containing the expression which creates visualisation.
    pub visualisation_module: String,
    #[allow(missing_docs)]
    pub expression: String
}

/// Used to enter deeper in the execution context stack. In general, all consequent stack items
/// should be `LocalCall`s.
#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct LocalCall {
    pub expression_id: ExpressionId
}

/// Points to a method definition.
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct MethodPointer {
    pub file: Path,
    pub defined_on_type: String,
    pub name: String
}

/// Used for entering a method. The first item on the execution context stack should always be
/// an `ExplicitCall`.
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct CapabilityRegistration {
    /// Method is the name of the capability listed in
    /// https://github.com/luna/enso/blob/master/doc/language-server/specification/enso-protocol.md#capabilities
    pub method: String,
    /// One of the enumerated `RegisterOptions` depending of `method`.
    pub register_options: RegisterOptions
}


// =======================
// === RegisterOptions ===
// =======================

/// `capability/acquire` takes method and options specific to the method. This type represents the
/// options. The used variant must match the method. See for details:
/// https://github.com/luna/enso/blob/master/doc/language-server/specification/enso-protocol.md#capabilities
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
#[allow(missing_docs)]
pub enum RegisterOptions {
    ReceivesTreeUpdates(ReceivesTreeUpdates),
    #[serde(rename_all = "camelCase")]
    ExecutionContextId { context_id: ContextId }
}

/// `RegisterOptions`' to receive file system tree updates.
#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct ReceivesTreeUpdates {
    pub path: Path
}
