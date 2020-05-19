use crate::prelude::*;

use crate::generated::binary_protocol_generated::org::enso::languageserver::protocol::binary as generated;
use generated::Path;
use generated::PathArgs;
use generated::OutboundMessage;
use generated::InboundPayload;
use generated::OutboundPayload;
use generated::InboundMessage;
use generated::InboundMessageArgs;
use generated::ReadFileCommand;
use generated::ReadFileCommandArgs;
use generated::InitSessionCommand;
use generated::InitSessionCommandArgs;
use generated::WriteFileCommand;
use generated::WriteFileCommandArgs;
use generated::EnsoUUID;

use crate::language_server::types::Path as LSPath;

use flatbuffers::FlatBufferBuilder;
use flatbuffers::UnionWIPOffset;
use flatbuffers::WIPOffset;
use crate::common::error::DeserializationError;
use crate::binary::message::{Message, FromServerOwned, FromServerRef, VisualisationContext, ToServerPayload, ToServerPayloadOwned};
use crate::generated::binary_protocol_generated::org::enso::languageserver::protocol::binary::{OutboundMessageArgs, Success, SuccessArgs, Error, ErrorArgs, FileContentsReply, FileContentsReplyArgs, VisualisationUpdate, VisualisationUpdateArgs, VisualisationContextArgs};

use crate::generated::binary_protocol_generated::org::enso::languageserver::protocol::binary as geneerated;

trait MessageBearer<'a> {
    type FbsMessage;
    type PayloadType;
}

trait SerializableTable<'a> : Sized {
    type Out : Sized;
    fn serialize_table(&self, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Self::Out>;
    fn deserialize_table(fbs:Self::Out) -> Result<Self,DeserializationError>;
    fn deserialize_opt_require(fbs:Option<Self::Out>) -> Result<Self, DeserializationError>{
        let missing_expected = || DeserializationError("Missing expected field".to_string());
        Self::deserialize_table(fbs.ok_or_else(missing_expected)?)
    }
}

impl<'a> SerializableTable<'a> for Vec<String> {
    type Out = flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<&'a str>>;

    fn serialize_table(&self, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Self::Out> {
        let strs = self.iter().map(|s| s.as_str()).collect_vec();
        builder.create_vector_of_strings(&strs)
    }

    fn deserialize_table(fbs: Self::Out) -> Result<Self, DeserializationError> {
        let indices = 0..fbs.len();
        Ok(indices.map(|ix| fbs.get(ix).to_string()).collect())
    }
}

impl<'a> SerializableTable<'a> for VisualisationContext {
    type Out = generated::VisualisationContext<'a>;
    fn serialize_table(&self, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Self::Out> {
        generated::VisualisationContext::create(builder, &VisualisationContextArgs {
            visualisationId : Some(&self.visualization_id.into()),
            expressionId    : Some(&self.expression_id.into()),
            contextId       : Some(&self.context_id.into()),
        })
    }

    fn deserialize_table(fbs:Self::Out) -> Result<Self,DeserializationError> {
        Ok(VisualisationContext {
            context_id       : fbs.contextId().into(),
            visualization_id : fbs.visualisationId().into(),
            expression_id    : fbs.expressionId().into(),
        })
    }
}

impl<'a> SerializableTable<'a> for LSPath {
    type Out = generated::Path<'a>;
    fn serialize_table(&self, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Self::Out> {
        let root_id      = self.root_id.into();
        let segments     = Vec::serialize_table(&self.segments,builder);
        Path::create(builder, &PathArgs {
            rootId   : Some(&root_id),
            segments : Some(segments),
        })
    }

    fn deserialize_table(fbs:Self::Out) -> Result<Self,DeserializationError> {
        let missing_root_id = || DeserializationError("Missing root ID".to_string());
        let root_id         = Uuid::from(fbs.rootId().ok_or_else(missing_root_id)?);
        let segments        = Vec::deserialize_opt_require(fbs.segments())?;
        Ok(LSPath {root_id,segments})
    }
}

/// Payload that can be serialized.
pub trait Serializable {
    fn write_message(&self, builder:&mut FlatBufferBuilder, correlation_id:Option<Uuid>, message_id:Uuid);
}

impl<T> Serializable for T where T:SerializableToServer {
    fn write_message(&self, builder:&mut FlatBufferBuilder, correlation_id:Option<Uuid>, message_id:Uuid) {
        let correlation_id = correlation_id.map(EnsoUUID::from);
        let message_id     = Some(message_id.into());
        self.write_message_fbs(builder, correlation_id.as_ref(), message_id.as_ref())
    }
}

/// Payloads that can be serialized and sent as a message to server.
pub trait SerializableToServer {
    /// Writes the message into a buffer and finishes it.
    fn write_message_fbs
    (&self
    , builder:&mut FlatBufferBuilder
    , correlationId:Option<&EnsoUUID>
    , messageId:Option<&EnsoUUID>) {
        let payload_type   = self.payload_type();
        let payload        = Some(self.write_payload(builder));
        let message        = InboundMessage::create(builder, &InboundMessageArgs {
            correlationId,
            messageId,
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
    (&self
     , builder:&mut FlatBufferBuilder
     , correlationId:Option<&EnsoUUID>
     , messageId:Option<&EnsoUUID>) {
        let payload_type   = self.payload_type();
        let payload        = Some(self.write_payload(builder));
        let message        = OutboundMessage::create(builder, &OutboundMessageArgs {
            correlationId,
            messageId,
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


impl FromServerOwned {
    pub fn deserialize_owned(message:&OutboundMessage) -> Result<Self,DeserializationError> {
        match message.payload_type() {
            OutboundPayload::ERROR => {
                let payload = message.payload_as_error().unwrap();
                Ok(FromServerOwned::Error {
                    code: payload.code(),
                    message: payload.message().unwrap_or_default().to_string(),
                })
            }
            OutboundPayload::FILE_CONTENTS_REPLY => {
                let payload = message.payload_as_file_contents_reply().unwrap();
                Ok(FromServerOwned::FileContentsReply {
                    contents: Vec::from(payload.contents().unwrap_or_default())
                })
            }
            OutboundPayload::SUCCESS => Ok(FromServerOwned::Success {}),
            OutboundPayload::VISUALISATION_UPDATE => {
                let payload = message.payload_as_visualisation_update().unwrap();
                let context = payload.visualisationContext();
                Ok(FromServerOwned::VisualizationUpdate {
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

impl SerializableFromServer for FromServerOwned {
    fn write_payload(&self, builder: &mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset> {
        match self {
            FromServerOwned::Success {} => {
                Success::create(builder, &SuccessArgs {}).as_union_value()
            }
            FromServerOwned::Error {code,message} => {
                let message  = builder.create_string(&message);
                Error::create(builder, &ErrorArgs {
                    code : *code,
                    message : Some(message),
                }).as_union_value()
            }
            FromServerOwned::FileContentsReply {contents} => {
                let contents = builder.create_vector(&contents);
                FileContentsReply::create(builder, &FileContentsReplyArgs {
                    contents : Some(contents)
                }).as_union_value()
            }
            FromServerOwned::VisualizationUpdate {data,context} => {
                let data = builder.create_vector(&data);
                let context = context.serialize_table(builder);
                VisualisationUpdate::create(builder, &VisualisationUpdateArgs {
                    data : Some(data),
                    visualisationContext : Some(context),
                }).as_union_value()
            }
        }
    }

    fn payload_type(&self) -> OutboundPayload {
        match self {
            FromServerOwned::Error {..}               => OutboundPayload::ERROR,
            FromServerOwned::Success {..}             => OutboundPayload::SUCCESS,
            FromServerOwned::FileContentsReply {..}   => OutboundPayload::FILE_CONTENTS_REPLY,
            FromServerOwned::VisualizationUpdate {..} => OutboundPayload::VISUALISATION_UPDATE,
        }
    }
}


impl<'a> FromServerRef<'a> {
    pub fn deserialize(message:&OutboundMessage<'a>) -> Result<Self,DeserializationError> {
        match message.payload_type() {
            OutboundPayload::ERROR => {
                let payload = message.payload_as_error().unwrap();
                Ok(FromServerRef::Error {
                    code: payload.code(),
                    message: payload.message().unwrap_or_default(),
                })
            }
            OutboundPayload::FILE_CONTENTS_REPLY => {
                let payload = message.payload_as_file_contents_reply().unwrap();
                Ok(FromServerRef::FileContentsReply {
                    contents: payload.contents().unwrap_or_default()
                })
            }
            OutboundPayload::SUCCESS => Ok(FromServerRef::Success {}),
            OutboundPayload::VISUALISATION_UPDATE => {
                let payload = message.payload_as_visualisation_update().unwrap();
                let context = payload.visualisationContext();
                Ok(FromServerRef::VisualizationUpdate {
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

/// Payloads that can be serialized and sent as a message to server.
pub trait DeserializableToServer : Sized {
    /// Writes the message into a buffer and finishes it.
    fn read_message(data:&[u8]) -> Result<Message<Self>,DeserializationError> {
        let message = flatbuffers::get_root::<InboundMessage>(data);
        let payload = Self::from_message(&message)?;
        Ok(Message {
            message_id : message.messageId().into(),
            correlation_id : message.correlationId().map(|id| id.into()),
            payload
        })
    }

    fn from_message(message:&InboundMessage) -> Result<Self,DeserializationError>;
}



impl<'a> SerializableToServer for ToServerPayload<'a> {
    fn write_payload(&self, builder: &mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset> {
        match self {
            ToServerPayload::InitSession {client_id} => {
                InitSessionCommand::create(builder, &InitSessionCommandArgs {
                    identifier : Some(&client_id.into())
                }).as_union_value()
            }
            ToServerPayload::WriteFile {path,contents} => {
                let path     = path.serialize_table(builder); //serialize_path(path,builder);
                let contents = builder.create_vector(contents);
                WriteFileCommand::create(builder, &WriteFileCommandArgs {
                    path : Some(path),
                    contents : Some(contents),
                }).as_union_value()
            }
            ToServerPayload::ReadFile {path} => {
                let path = path.serialize_table(builder);//serialize_path(path,builder);
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

impl DeserializableToServer for ToServerPayloadOwned {
    fn from_message(message: &InboundMessage) -> Result<Self,DeserializationError> {
        let missing_required_field = DeserializationError("missing a required field".into());
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
                    path: LSPath::deserialize_opt_require(payload.path())?,
                    contents: Vec::from(payload.contents().unwrap_or_default())
                })
            }
            InboundPayload::READ_FILE_CMD => {
                let payload = message.payload_as_write_file_cmd().unwrap();
                Ok(ToServerPayloadOwned::ReadFile {
                    path: LSPath::deserialize_opt_require(payload.path())?,
                })
            }
            InboundPayload::NONE =>
                Err(DeserializationError("Received a message without payload. This is not allowed, \
                                         according to the spec.".into()))
        }
    }
}


