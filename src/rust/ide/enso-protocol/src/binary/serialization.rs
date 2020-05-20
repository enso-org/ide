//! Code for converting between FlatBuffer-generated wrappers and our representation of the protocol
//! messages and their parts.

use crate::prelude::*;

use crate::generated::binary_protocol_generated as generated_root;
use generated_root::org::enso::languageserver::protocol::binary as generated;
use generated::EnsoUUID;
use generated::Error;
use generated::ErrorArgs;
use generated::FileContentsReply;
use generated::FileContentsReplyArgs;
use generated::InboundPayload;
use generated::InboundMessage;
use generated::InboundMessageArgs;
use generated::InitSessionCommand;
use generated::InitSessionCommandArgs;
use generated::OutboundMessage;
use generated::OutboundMessageArgs;
use generated::OutboundPayload;
use generated::Path;
use generated::PathArgs;
use generated::ReadFileCommand;
use generated::ReadFileCommandArgs;
use generated::Success;
use generated::SuccessArgs;
use generated::VisualisationContextArgs;
use generated::VisualisationUpdate;
use generated::VisualisationUpdateArgs;
use generated::WriteFileCommand;
use generated::WriteFileCommandArgs;
use crate::common::error::DeserializationError;
use crate::binary::message::Message;
use crate::binary::message::FromServerPayloadOwned;
use crate::binary::message::FromServerPayload;
use crate::binary::message::VisualisationContext;
use crate::binary::message::ToServerPayload;
use crate::binary::message::ToServerPayloadOwned;
use crate::language_server::types::Path as LSPath;

use flatbuffers::FlatBufferBuilder;
use flatbuffers::UnionWIPOffset;
use flatbuffers::WIPOffset;



// ==========================
// === SerializableObject ===
// ==========================


// === Trait ===

/// All entities that can be serialized to the FlatBuffers and represented as offsets.
///
/// Supports both serialization and deserialization.
trait SerializableObject<'a> : Sized {
    /// The FlatBuffer's generated type for this type representation.
    type Out : Sized;

    /// Writes this table to the buffer and returns its handle.
    fn serialize(&self, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Self::Out>;

    /// Instantiates Self and reads the data from the FlatBuffers representation.
    fn deserialize(fbs:Self::Out) -> Result<Self,DeserializationError>;

    /// Instantiates Self and reads the data from the optional FlatBuffers representation.
    /// Will fail always if the representation is not present.
    fn deserialize_required_opt(fbs:Option<Self::Out>) -> Result<Self, DeserializationError>{
        let missing_expected = || DeserializationError("Missing expected field".to_string());
        Self::deserialize(fbs.ok_or_else(missing_expected)?)
    }
}


// === impl Vec<String> ===

impl<'a> SerializableObject<'a> for Vec<String> {
    type Out = flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<&'a str>>;

    fn serialize(&self, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Self::Out> {
        let strs = self.iter().map(|s| s.as_str()).collect_vec();
        builder.create_vector_of_strings(&strs)
    }

    fn deserialize(fbs: Self::Out) -> Result<Self, DeserializationError> {
        let indices = 0..fbs.len();
        Ok(indices.map(|ix| fbs.get(ix).to_string()).collect())
    }
}


// === impl VisualisationContext ===

impl<'a> SerializableObject<'a> for VisualisationContext {
    type Out = generated::VisualisationContext<'a>;
    fn serialize(&self, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Self::Out> {
        generated::VisualisationContext::create(builder, &VisualisationContextArgs {
            visualisationId : Some(&self.visualization_id.into()),
            expressionId    : Some(&self.expression_id.into()),
            contextId       : Some(&self.context_id.into()),
        })
    }

    fn deserialize(fbs:Self::Out) -> Result<Self,DeserializationError> {
        Ok(VisualisationContext {
            context_id       : fbs.contextId().into(),
            visualization_id : fbs.visualisationId().into(),
            expression_id    : fbs.expressionId().into(),
        })
    }
}


// === impl language server's Path ===

impl<'a> SerializableObject<'a> for LSPath {
    type Out = Path<'a>;
    fn serialize(&self, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Self::Out> {
        let root_id      = self.root_id.into();
        let segments     = Vec::serialize(&self.segments, builder);
        Path::create(builder, &PathArgs {
            rootId   : Some(&root_id),
            segments : Some(segments),
        })
    }

    fn deserialize(fbs:Self::Out) -> Result<Self,DeserializationError> {
        let missing_root_id = || DeserializationError("Missing root ID".to_string());
        let root_id         = Uuid::from(fbs.rootId().ok_or_else(missing_root_id)?);
        let segments        = Vec::deserialize_required_opt(fbs.segments())?;
        Ok(LSPath {root_id,segments})
    }
}



// =========================
// === SerializableUnion ===
// =========================

pub trait SerializableUnionMember<'a, ParentType:'a> : Sized {
    type EnumType;

    fn serialize(&self, builder: &mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset>;
    fn active_variant(&self) -> Self::EnumType;

    fn deserialize(parent:ParentType) -> Self;
}


// impl<'a> SerializablePayloadToServer for ToServerPayload<'a> {
//     fn write_payload(&self, builder:&mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset> {
//         match self {
//             ToServerPayload::InitSession {client_id} => {
//                 InitSessionCommand::create(builder, &InitSessionCommandArgs {
//                     identifier : Some(&client_id.into())
//                 }).as_union_value()
//             }
//             ToServerPayload::WriteFile {path,contents} => {
//                 let path     = path.serialize(builder); //serialize_path(path,builder);
//                 let contents = builder.create_vector(contents);
//                 WriteFileCommand::create(builder, &WriteFileCommandArgs {
//                     path : Some(path),
//                     contents : Some(contents),
//                 }).as_union_value()
//             }
//             ToServerPayload::ReadFile {path} => {
//                 let path = path.serialize(builder);//serialize_path(path,builder);
//                 ReadFileCommand::create(builder, &ReadFileCommandArgs {
//                     path : Some(path)
//                 }).as_union_value()
//             }
//         }
//     }
//
//     fn payload_type(&self) -> InboundPayload {
//         match self {
//             ToServerPayload::InitSession {..} => InboundPayload::INIT_SESSION_CMD,
//             ToServerPayload::WriteFile   {..} => InboundPayload::WRITE_FILE_CMD,
//             ToServerPayload::ReadFile    {..} => InboundPayload::READ_FILE_CMD,
//         }
//     }
// }



// ==========================
// === SerializableObject ===
// ==========================

/// Payload that can be serialized as a message part.
pub trait SerializableInMessage {
    /// Serializes the message with this payload to the builder and calls `finish` on it.
    fn write_message(&self, builder:&mut FlatBufferBuilder, correlation_id:Option<Uuid>, message_id:Uuid);
}

/// Payloads that can be serialized and sent as a message to server.
///
/// Abstracts over `ToServerPayloadOwned` ``
pub trait SerializablePayloadToServer {
    /// Writes the message into a buffer and finishes it.
    fn write_message_fbs
    (&self
    , builder:&mut FlatBufferBuilder
    , correlation_id:Option<Uuid>
    , message_id:Uuid) {
        let correlation_id = correlation_id.map(EnsoUUID::from);
        let message_id     = message_id.into();
        let payload_type   = self.payload_type();
        let payload        = Some(self.write_payload(builder));
        let message        = InboundMessage::create(builder, &InboundMessageArgs {
            correlationId : correlation_id.as_ref(),
            messageId     : Some(&message_id),
            payload_type,
            payload,
        });
        builder.finish(message,None);
    }

    /// Writes just the payload into the buffer.
    fn write_payload(&self, builder:&mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset>;

    /// Returns enumeration describing variant of this payload.
    fn payload_type(&self) -> InboundPayload;
}

/// Payloads that can be serialized and sent as a message to server.
pub trait SerializableFromServer  {
    /// Writes the message into a buffer and finishes it.
    fn write_message_fbs
    ( &self
    , builder:&mut FlatBufferBuilder
    , correlation_id:Option<Uuid>
    , message_id:Uuid) {
        let correlation_id = correlation_id.map(EnsoUUID::from);
        let message_id     = message_id.into();
        let payload_type   = self.payload_type();
        let payload        = Some(self.write_payload(builder));
        let message        = OutboundMessage::create(builder, &OutboundMessageArgs {
            correlationId : correlation_id.as_ref(),
            messageId     : Some(&message_id),
            payload_type,
            payload,
        });
        builder.finish(message,None);
    }

    /// Writes just the payload into the buffer.
    fn write_payload(&self, builder:&mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset>;

    /// Returns enumeration describing variant of this payload.
    fn payload_type(&self) -> OutboundPayload;
}


impl FromServerPayloadOwned {
    /// Deserializes this payload from FlatBuffer's message from server representation.
    pub fn deserialize_owned(message:&OutboundMessage) -> Result<Self,DeserializationError> {
        match message.payload_type() {
            OutboundPayload::ERROR => {
                let payload = message.payload_as_error().unwrap();
                Ok(FromServerPayloadOwned::Error {
                    code: payload.code(),
                    message: payload.message().unwrap_or_default().to_string(),
                })
            }
            OutboundPayload::FILE_CONTENTS_REPLY => {
                let payload = message.payload_as_file_contents_reply().unwrap();
                Ok(FromServerPayloadOwned::FileContentsReply {
                    contents: Vec::from(payload.contents().unwrap_or_default())
                })
            }
            OutboundPayload::SUCCESS => Ok(FromServerPayloadOwned::Success {}),
            OutboundPayload::VISUALISATION_UPDATE => {
                let payload = message.payload_as_visualisation_update().unwrap();
                let context = payload.visualisationContext();
                Ok(FromServerPayloadOwned::VisualizationUpdate {
                    data: Vec::from(payload.data()),
                    context: VisualisationContext {
                        context_id: context.contextId().into(),
                        expression_id: context.expressionId().into(),
                        visualization_id: context.visualisationId().into(),
                    }
                })
            }
            OutboundPayload::NONE =>
                Err(DeserializationError("Received a message without payload. This is not allowed, \
                                         according to the spec.".into()))
        }
    }
}

impl SerializableFromServer for FromServerPayloadOwned {
    fn write_payload(&self, builder: &mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset> {
        match self {
            FromServerPayloadOwned::Success {} => {
                Success::create(builder, &SuccessArgs {}).as_union_value()
            }
            FromServerPayloadOwned::Error {code,message} => {
                let message  = builder.create_string(&message);
                Error::create(builder, &ErrorArgs {
                    code : *code,
                    message : Some(message),
                }).as_union_value()
            }
            FromServerPayloadOwned::FileContentsReply {contents} => {
                let contents = builder.create_vector(&contents);
                FileContentsReply::create(builder, &FileContentsReplyArgs {
                    contents : Some(contents)
                }).as_union_value()
            }
            FromServerPayloadOwned::VisualizationUpdate {data,context} => {
                let data = builder.create_vector(&data);
                let context = context.serialize(builder);
                VisualisationUpdate::create(builder, &VisualisationUpdateArgs {
                    data : Some(data),
                    visualisationContext : Some(context),
                }).as_union_value()
            }
        }
    }

    fn payload_type(&self) -> OutboundPayload {
        match self {
            FromServerPayloadOwned::Error {..}               => OutboundPayload::ERROR,
            FromServerPayloadOwned::Success {..}             => OutboundPayload::SUCCESS,
            FromServerPayloadOwned::FileContentsReply {..}   => OutboundPayload::FILE_CONTENTS_REPLY,
            FromServerPayloadOwned::VisualizationUpdate {..} => OutboundPayload::VISUALISATION_UPDATE,
        }
    }
}

impl<'a> SerializableInMessage for FromServerPayloadOwned {
    fn write_message(&self, builder:&mut FlatBufferBuilder, correlation_id:Option<Uuid>, message_id:Uuid) {
        self.write_message_fbs(builder,correlation_id,message_id)
    }
}

impl<'a> FromServerPayload<'a> {
    /// Deserializes this payload from FlatBuffer's message from server representation.
    pub fn deserialize(message:&OutboundMessage<'a>) -> Result<Self,DeserializationError> {
        match message.payload_type() {
            OutboundPayload::ERROR => {
                let payload = message.payload_as_error().unwrap();
                Ok(FromServerPayload::Error {
                    code: payload.code(),
                    message: payload.message().unwrap_or_default(),
                })
            }
            OutboundPayload::FILE_CONTENTS_REPLY => {
                let payload = message.payload_as_file_contents_reply().unwrap();
                Ok(FromServerPayload::FileContentsReply {
                    contents: payload.contents().unwrap_or_default()
                })
            }
            OutboundPayload::SUCCESS => Ok(FromServerPayload::Success {}),
            OutboundPayload::VISUALISATION_UPDATE => {
                let payload = message.payload_as_visualisation_update().unwrap();
                let context = payload.visualisationContext();
                Ok(FromServerPayload::VisualizationUpdate {
                    data: payload.data(),
                    context: VisualisationContext {
                        context_id: context.contextId().into(),
                        expression_id: context.expressionId().into(),
                        visualization_id: context.visualisationId().into(),
                    }
                })
            }
            OutboundPayload::NONE =>
                Err(DeserializationError("Received a message without payload. This is not allowed, \
                                         according to the spec.".into()))
        }
    }
}

/// Payloads that can be deserialized from the data received from server.
pub trait DeserializableToServer : Sized {
    /// Deserializes the message (with Self payload type) from the binary data.
    fn read_message(data:&[u8]) -> Result<Message<Self>,DeserializationError> {
        let message = flatbuffers::get_root::<InboundMessage>(data);
        let payload = Self::from_message(&message)?;
        Ok(Message {
            message_id : message.messageId().into(),
            correlation_id : message.correlationId().map(|id| id.into()),
            payload
        })
    }

    /// Retrieves the payload data from the FlatBuffer representation of the message.
    fn from_message(message:&InboundMessage) -> Result<Self,DeserializationError>;
}

impl<'a> SerializablePayloadToServer for ToServerPayload<'a> {
    fn write_payload(&self, builder:&mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset> {
        match self {
            ToServerPayload::InitSession {client_id} => {
                InitSessionCommand::create(builder, &InitSessionCommandArgs {
                    identifier : Some(&client_id.into())
                }).as_union_value()
            }
            ToServerPayload::WriteFile {path,contents} => {
                let path     = path.serialize(builder); //serialize_path(path,builder);
                let contents = builder.create_vector(contents);
                WriteFileCommand::create(builder, &WriteFileCommandArgs {
                    path : Some(path),
                    contents : Some(contents),
                }).as_union_value()
            }
            ToServerPayload::ReadFile {path} => {
                let path = path.serialize(builder);//serialize_path(path,builder);
                ReadFileCommand::create(builder, &ReadFileCommandArgs {
                    path : Some(path)
                }).as_union_value()
            }
        }
    }

    fn payload_type(&self) -> InboundPayload {
        match self {
            ToServerPayload::InitSession {..} => InboundPayload::INIT_SESSION_CMD,
            ToServerPayload::WriteFile   {..} => InboundPayload::WRITE_FILE_CMD,
            ToServerPayload::ReadFile    {..} => InboundPayload::READ_FILE_CMD,
        }
    }
}

impl<'a> SerializableInMessage for ToServerPayload<'a> {
    fn write_message(&self, builder:&mut FlatBufferBuilder, correlation_id:Option<Uuid>, message_id:Uuid) {
        self.write_message_fbs(builder,correlation_id,message_id)
    }
}

impl DeserializableToServer for ToServerPayloadOwned {
    fn from_message(message: &InboundMessage) -> Result<Self,DeserializationError> {
        match message.payload_type() {
            InboundPayload::INIT_SESSION_CMD => {
                let payload = message.payload_as_init_session_cmd().unwrap();
                Ok(ToServerPayloadOwned::InitSession {
                    client_id: payload.identifier().into()
                })
            }
            InboundPayload::WRITE_FILE_CMD => {
                let payload = message.payload_as_write_file_cmd().unwrap();

                Ok(ToServerPayloadOwned::WriteFile {
                    path: LSPath::deserialize_required_opt(payload.path())?,
                    contents: Vec::from(payload.contents().unwrap_or_default())
                })
            }
            InboundPayload::READ_FILE_CMD => {
                let payload = message.payload_as_read_file_cmd().unwrap();
                Ok(ToServerPayloadOwned::ReadFile {
                    path: LSPath::deserialize_required_opt(payload.path())?,
                })
            }
            InboundPayload::NONE =>
                Err(DeserializationError("Received a message without payload. This is not allowed, \
                                         according to the spec.".into()))
        }
    }
}
