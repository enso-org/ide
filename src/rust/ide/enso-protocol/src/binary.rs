//! Crate containing the Engine Services binary protocol interface.

pub mod connection;
pub mod uuid;

#[allow(dead_code, unused_imports)]
use flatbuffers;

use crate::prelude::*;

use futures::{FutureExt, SinkExt};
use futures::StreamExt;
use futures::Stream;
use futures::channel::mpsc::unbounded;
use futures::channel::mpsc::UnboundedSender;
use futures::channel::oneshot;
use logger::*;

pub use crate::generated::binary_protocol_generated as binary_protocol;
use json_rpc::{Transport, TransportEvent};
use crate::generated::binary_protocol_generated::org::enso::languageserver::protocol::binary::EnsoUUID;
use crate::language_server::Path as LSPath;
use binary_protocol::org::enso::languageserver::protocol::binary::*;
use flatbuffers::{FlatBufferBuilder, WIPOffset, UnionWIPOffset};
use crate::generated::binary_protocol_generated::org::enso::languageserver::protocol::binary::InboundPayload::INIT_SESSION_CMD;
use std::future::Future;
use utils::fail::FallibleResult;
use json_rpc::error::{RpcError, HandlingError};
use futures::channel::oneshot::Canceled;
use std::pin::Pin;

#[derive(Clone,Debug)]
pub struct VisualisationContext {
    pub visualization_id : Uuid,
    pub context_id       : Uuid,
    pub expression_id    : Uuid,
}

pub fn serialize_path<'a>(path:&LSPath, builder:&mut FlatBufferBuilder<'a>) -> WIPOffset<Path<'a>> {
    let root_id      = path.root_id.into();
    let segment_refs = path.segments.iter().map(|s| s.as_str()).collect_vec();
    let segments     = builder.create_vector_of_strings(&segment_refs);
    Path::create(builder, &PathArgs {
        rootId   : Some(&root_id),
        segments : Some(segments),
    })
}

#[derive(Clone,Debug)]
pub enum FromServerOwned {
    Error {code:i32, message:String},
    Success {},
    VisualizationUpdate {context:VisualisationContext, data:Vec<u8>},
    FileContentsReply   {contents:Vec<u8>},
}

impl FromServerOwned {
    pub fn deserialize<'a>(message:&OutboundMessage<'a>) -> Self {
        match message.payload_type() {
            OutboundPayload::ERROR => {
                let payload = message.payload_as_error().unwrap();
                FromServerOwned::Error {
                    code: payload.code(),
                    message: payload.message().unwrap_or_default().to_string(),
                }
            }
            OutboundPayload::FILE_CONTENTS_REPLY => {
                let payload = message.payload_as_file_contents_reply().unwrap();
                FromServerOwned::FileContentsReply {
                    contents: Vec::from(payload.contents().unwrap_or_default())
                }
            }
            OutboundPayload::SUCCESS => FromServerOwned::Success {},
            OutboundPayload::VISUALISATION_UPDATE => {
                let payload = message.payload_as_visualisation_update().unwrap();
                let context = payload.visualisationContext();
                FromServerOwned::VisualizationUpdate {
                    data: Vec::from(payload.data()),
                    context: VisualisationContext {
                        context_id: context.contextId().into(),
                        expression_id: context.expressionId().into(),
                        visualization_id: context.visualisationId().into(),
                    }
                }
            }
            _ => todo!()
        }
    }
}

#[derive(Clone,Debug)]
pub enum FromServer<'a> {
    Error {code:i32, message:&'a str},
    Success {},
    VisualizationUpdate {context:VisualisationContext, data:&'a [u8]},
    FileContentsReply {contents:&'a [u8]},
}

#[derive(Clone,Debug)]
pub enum ToServerPayload<'a> {
    InitSession {client_id:Uuid},
    WriteFile   {path:&'a LSPath, contents:&'a[u8]},
    ReadFile    {path:&'a LSPath}
}

/// Common message envelope for binary protocol.
///
/// `T` should represent the payload.
#[derive(Clone,Debug)]
pub struct Message<T> {
    pub message_id     : Uuid,
    pub correlation_id : Option<Uuid>,
    pub payload        : T,
}

impl<T> Message<T> {
    pub fn new(payload:T) -> Message<T> {
        Message {
            message_id     : Uuid::new_v4(),
            correlation_id : None,
            payload,
        }
    }
}

/// When payload supports serialization, we can serialize the whole message.
impl<T: IsPayloadToServer> Message<T> {
    pub fn write_message(&self, builder:&mut FlatBufferBuilder) {
        self.payload.write_message(builder,self.message_id,self.correlation_id)
    }

    pub fn build_buffer(&self) -> FlatBufferBuilder {
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
        self.write_message(&mut builder);
        builder
    }

    pub fn with_message<R>(&self, f:impl FnOnce(&[u8]) -> R) -> R {
        let buffer = self.build_buffer();
        let data = buffer.finished_data();
        f(data)
    }
}

impl Message<FromServerOwned> {
    pub fn deserialize_owned(data:&[u8]) -> Self {
        let message = flatbuffers::get_root::<OutboundMessage>(data);
        let payload = FromServerOwned::deserialize(&message);

        let enso = message.correlationId();
        let uuid = enso.map(Uuid::from);
        println!("From {:?} we got {:?}", enso, uuid);
        Message {
            message_id     : message.messageId().into(),
            correlation_id : message.correlationId().map(|id| id.into()),
            payload
        }
    }
}

type MessageFromServerOwned = Message<FromServerOwned>;


pub trait IsPayloadToServer {
    type PayloadType;

    fn write_message(&self, builder:&mut FlatBufferBuilder, message_id:Uuid, correlation_id:Option<Uuid>);
    fn write_payload(&self, builder:&mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset>;
    fn payload_type(&self) -> Self::PayloadType;
}

impl<'a> IsPayloadToServer for ToServerPayload<'a> {
    type PayloadType = InboundPayload;

    fn write_message(&self, builder:&mut FlatBufferBuilder, message_id:Uuid, correlation_id:Option<Uuid>) {
        let payload_type   = self.payload_type();
        let payload        = Some(self.write_payload(builder));
        let correlation_id2 = correlation_id.map(EnsoUUID::from);
        println!("Sending message id: {:?}, generated from {}", EnsoUUID::from(message_id), message_id);
        let message        = InboundMessage::create(builder, &InboundMessageArgs {
            correlationId : correlation_id2.as_ref(),
            messageId     : Some(&message_id.into()),
            payload_type,
            payload,
        });
        builder.finish(message,None);
    }

    fn write_payload(&self, builder: &mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset> {
        match self {
            ToServerPayload::InitSession {client_id} => {
                InitSessionCommand::create(builder, &InitSessionCommandArgs {
                    identifier : Some(&client_id.into())
                }).as_union_value()
            }
            ToServerPayload::WriteFile {path,contents} => {
                let path     = serialize_path(path,builder);
                let contents = builder.create_vector(contents);
                WriteFileCommand::create(builder, &WriteFileCommandArgs {
                    path : Some(path),
                    contents : Some(contents),
                }).as_union_value()
            }
            ToServerPayload::ReadFile {path} => {
                let path = serialize_path(path,builder);
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

/// When trying to parse a line, not a single line was produced.
#[derive(Debug,Fail,Clone,Copy)]
#[fail(display = "No active request by id {}", _0)]
pub struct NoSuchRequest<Id : Sync + Send + Debug + Display + 'static>(pub Id);


#[derive(Derivative,CloneRef,Debug,Default)]
#[derivative(Clone(bound=""))]
pub struct RequestHandler<Id,Reply>
where Id:Hash+Eq {
    ongoing_calls : Rc<RefCell<HashMap<Id,oneshot::Sender<Reply>>>>,
}

impl<Id,Reply> RequestHandler<Id,Reply>
    where Id:Hash+Eq {
    pub fn new() -> RequestHandler<Id,Reply> {
        RequestHandler {
            ongoing_calls : Rc::new(RefCell::new(default()))
        }
    }

    pub fn remove_request(&self, id:&Id) -> Option<oneshot::Sender<Reply>> {
        with(self.ongoing_calls.borrow_mut(), |mut map| map.remove(id))
    }

    pub fn open_request(&self, id:Id, sender:oneshot::Sender<Reply>) {
        with(self.ongoing_calls.borrow_mut(), |mut map| {
            map.insert(id,sender);
        })
    }

    pub fn clear(&self) {
        with(self.ongoing_calls.borrow_mut(), |mut map| map.clear())
    }

    pub fn complete_request(&self, id:Id, reply:Reply) -> FallibleResult<()>
        where Id : Display + Debug + Send + Sync + 'static {
        if let Some(mut request) = self.remove_request(&id) {
            // Explicitly ignore error. Can happen only if the other side already dropped future
            // with the call result. In such case no one needs to be notified and we are fine.
            let _ = request.send(reply);
            Ok(())
        } else {
            Err(NoSuchRequest(id).into())
        }
    }
}

fn open_request<SendReq,H,F,R>(handler:H, id:H::Id, f:F, send_request:SendReq) -> impl Future<Output = FallibleResult<R>>
    where
        H : HandlerLike,
        F : FnOnce(H::Reply) -> FallibleResult<R>,
        SendReq : FnOnce() -> FallibleResult<()> {
    let (sender, receiver) = oneshot::channel::<H::Reply>();
    let ret                = receiver.map(|result_or_cancel| {
        let result = result_or_cancel?;
        f(result)
    });

    handler.ongoing_calls().open_request(id,sender);
    if send_request().is_err() {
        handler.ongoing_calls().remove_request(&id);
    }
    ret
}


fn transport_event_stream(transport:&mut dyn Transport) -> impl Stream<Item = TransportEvent> {
    let (event_transmitter, event_receiver) = unbounded();
    transport.set_event_transmitter(event_transmitter);
    event_receiver
}

type Runner<H> = impl Future<Output = ()>;

pub trait HandlerLike : Clone  + 'static {
    type Id : Copy + Debug + Display + Hash + Eq + Send + Sync + 'static;
    //type Request : Debug;
    type Reply : Debug ;
    type Notification : Debug;

    fn logger(&self) -> &Logger;
    fn borrow_mut_event_sender(&self) -> Option<RefMut<UnboundedSender<Event<Self::Notification>>>>;
    fn borrow_mut_transport(&self) -> RefMut<dyn Transport>;
    fn ongoing_calls(&self) -> &RequestHandler<Self::Id,Self::Reply>;
    fn process_event(&self, event:TransportEvent);

    fn emit_event(&self, event:Event<Self::Notification>) {
        if let Some(mut sender) = self.borrow_mut_event_sender() {
            sender.send(event);
        }
    }

    fn with_transport<R>(&self, f:impl FnOnce(&mut dyn Transport) -> R) -> R {
        let mut transport = self.borrow_mut_transport();
        f(transport.deref_mut())
    }

    fn send_text_message(&self, data:&str) -> FallibleResult<()> {
        self.with_transport(|t| t.send_text(data))
    }

    fn send_binary_message(&self, data:&[u8]) -> FallibleResult<()> {
        self.with_transport(|t| t.send_binary(data))
    }

    fn process_reply(&self, id:Self::Id, reply:Self::Reply) {
        info!(self.logger(),"Processing reply to request {id}");
        if let Err(error) = self.ongoing_calls().complete_request(id,reply) {
            self.emit_error(error);
        }
    }
    fn emit_notification(&self, notification:Self::Notification) {
        info!(self.logger(),"Emitting notification: {notification:?}");
        let event = Event::Notification(notification);
        self.emit_event(event);
    }

    fn emit_error(&self, error:impl Into<failure::Error> + Debug) {
        info!(self.logger(),"Emitting error: {error:?}");
        let event = Event::Error(error.into());
        self.emit_event(event);
    }

    fn open_request(&self, id:Self::Id, completer:oneshot::Sender<Self::Reply>) {
        info!(self.logger(),"Opening request: {id}");
        self.ongoing_calls().open_request(id,completer)
    }

    fn runner(&self) -> Runner<Self> {
        let event_receiver = self.with_transport(|t| t.establish_event_stream());
        let this = self.clone();
        event_receiver.for_each(move |event: TransportEvent| {
            info!(this.logger(), "Transport event received: {event:?}");
            this.process_event(event);
            futures::future::ready(())
        })
    }
}

/// Event emitted by the `Handler<N>`.
#[derive(Debug)]
pub enum Event<N> {
    /// Transport has been closed.
    Closed,
    /// Error occurred.
    Error(failure::Error),
    /// Notification received.
    Notification(N),
}

#[derive(Clone,Debug)]
pub enum Notification {
    VisualizationUpdate {context:VisualisationContext, data:Vec<u8>},
}

#[derive(Clone,Derivative)]
#[derivative(Debug)]
pub struct Client {
    logger          : Logger,
    transport       : Rc<RefCell<dyn Transport>>,
    requests        : RequestHandler<Uuid,FromServerOwned>,
    #[derivative(Debug="ignore")]
    outgoing_events : RefCell<Option<UnboundedSender<Event<Notification>>>>,
}

impl HandlerLike for Client {
    type Id = Uuid;
    //type Request = To
    type Reply = FromServerOwned;
    type Notification = Notification;

    fn process_event(&self, event:TransportEvent) {
        if let TransportEvent::BinaryMessage(data) = event {
            let message = MessageFromServerOwned::deserialize_owned(&data);
            info!(self.logger,"Received binary message {message:?}");
            match message.payload {
                FromServerOwned::VisualizationUpdate {context,data} =>
                    self.emit_notification(Notification::VisualizationUpdate {data,context}),
                _ => {
                    if let Some(correlation_id) = message.correlation_id {
                        self.process_reply(correlation_id, message.payload)
                    } else {
                        // Not a known notification and yet not a response to our request.
                    }
                }
            }
        }
    }

    fn borrow_mut_transport(&self) -> RefMut<dyn Transport> {
        self.transport.borrow_mut()
    }

    fn ongoing_calls(&self) -> &RequestHandler<Uuid, Self::Reply> {
        &self.requests
    }

    fn logger(&self) -> &Logger {
        &self.logger
    }

    fn borrow_mut_event_sender(&self) -> Option<RefMut<UnboundedSender<Event<Self::Notification>>>> {
        let refmut = self.outgoing_events.borrow_mut();
        let is_present = refmut.is_some();
        if is_present {
            Some(RefMut::map(refmut, |s| s.as_mut().unwrap()))
        } else {
            None
        }
    }
}

pub fn expect_success(result:FromServerOwned) -> FallibleResult<()> {
    match result {
        FromServerOwned::Success {} => Ok(()),
        _ => Err(RpcError::MismatchedResponseType.into()),
    }
}

impl Client {
    pub fn new(mut transport:impl Transport + 'static) -> Client {
        Client {
            logger          : Logger::new("BinaryProtocolClient"),
            transport       : Rc::new(RefCell::new(transport)),
            requests        : RequestHandler::new(),
            outgoing_events : default(),
        }
    }


    pub fn open<F,R>(&self, payload: ToServerPayload, f:F) -> impl Future<Output=FallibleResult<R>>
    where F: FnOnce(FromServerOwned) -> FallibleResult<R> {
        let message = Message::new(payload);
        info!(self.logger,"Sending binary message {message:?}");
        message.with_message(|data| self.send_binary_message(data));
        //self.send_binary_message(message.build_buffer().finished_data());
        let (sender, receiver) = oneshot::channel::<FromServerOwned>();
        let logger = self.logger.clone_ref();
        let ret = receiver.map(move |result_or_cancel| {
            // Deal with cancel and peer-provided error.
            info!(logger,"Processing request reply {result_or_cancel:?}");
            let result = result_or_cancel?;
            if let FromServerOwned::Error {code,message} = result {
                let error = RpcError::new_remote_error(code.into(), message);
                Err(error.into())
            } else {
                f(result)
            }
        });
        self.open_request(message.message_id, sender);
        ret
    }

    pub fn init(&self, client_id:Uuid) -> impl Future<Output = FallibleResult<()>> {
        info!(self.logger,"Initializing binary connection as {client_id}");
        let payload = ToServerPayload::InitSession {client_id};
        self.open(payload,expect_success)
    }

    pub fn write_file(&self, path:&LSPath, contents:&[u8]) -> impl Future<Output = FallibleResult<()>> {
        info!(self.logger,"Writing file {path} with {contents:?}");
        let payload = ToServerPayload::WriteFile {path,contents};
        self.open(payload,expect_success)
    }

    pub fn read_file(&self, path:&LSPath) -> impl Future<Output = FallibleResult<Vec<u8>>> {
        info!(self.logger,"Reading file {path}");
        let payload = ToServerPayload::ReadFile {path};
        self.open(payload, move |result| {
            match result {
                FromServerOwned::FileContentsReply {contents} =>
                    Ok(contents),
                _ =>
                    Err(RpcError::MismatchedResponseType.into()),
            }
        })
    }

    pub fn handle_response(&self, correlation_id:Uuid, payload:FromServerOwned) {
        self.ongoing_calls().complete_request(correlation_id,payload);
    }

    // pub fn runner(&self) -> impl Future<Output = ()> {
    //     let event_receiver = self.transport.borrow_mut().establish_event_stream();
    //     let this = self.clone();
    //     event_receiver.for_each(move |event:TransportEvent| {
    //         info!(this.logger,"Transport event received: {event:?}");
    //         this.process_event(event);
    //         futures::future::ready(())
    //     })
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test_configure;
    wasm_bindgen_test_configure!(run_in_browser);


    #[test]
    fn uuid_round_trips() {
        //let uuid = Uuid::new_v4();
        let uuid = Uuid::parse_str("6de39f7b-df3a-4a3c-84eb-5eaf96ddbac2").unwrap();
        println!("uuid bytes: {:?}", uuid.as_bytes());
        println!("initial uuid: {:?}", uuid);
        let enso = EnsoUUID::from(uuid);
        println!("enso-uuid: {:?}", enso);

        let uuid2 = Uuid::from(enso);
        println!("restored uuid: {:?}", uuid2);

        let enso_uuid = EnsoUUID::from(uuid);
        println!("uuid bytes: {:?}", enso_uuid.leastSigBits().to_le_bytes());

        assert_eq!(uuid,Uuid::from(EnsoUUID::from(uuid)));
    }


    #[wasm_bindgen_test::wasm_bindgen_test(async)]
    #[allow(dead_code)]
    async fn first_real_test() {
        ensogl_system_web::set_stdout();



        assert!(false);
    }
}