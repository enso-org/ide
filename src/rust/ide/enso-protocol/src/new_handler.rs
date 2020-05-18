use crate::prelude::*;
use json_rpc::{Transport, TransportEvent};
use futures::channel::mpsc::UnboundedSender;
use futures::channel::oneshot;
use futures::channel::oneshot::Canceled;
use futures::SinkExt;

use logger::*;
use std::future::Future;
use crate::binary::NoSuchRequest;
use utils::fail::FallibleResult;
use json_rpc::error::RpcError;

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


#[derive(Debug,Default)]
pub struct RequestHandler<Id,Reply> where Id:Hash+Eq {
    logger        : Logger,
    ongoing_calls : HashMap<Id,oneshot::Sender<Reply>>,
}

impl<Id,Reply> RequestHandler<Id,Reply>
where Id:Copy + Debug + Display + Hash + Eq + Send + Sync + 'static {
    pub fn new(parent_logger:&Logger) -> RequestHandler<Id,Reply> {
        RequestHandler {
            logger : parent_logger.sub("ongoing_calls"),
            ongoing_calls : HashMap::new(),
        }
    }

    pub fn remove_request(&mut self, id:&Id) -> Option<oneshot::Sender<Reply>> {
        let ret = self.ongoing_calls.remove(id);
        if ret.is_some() {
            info!(self.logger,"Removing request {id}");
        } else {
            info!(self.logger,"Failed to remove non-present request {id}");
        }
        ret
    }

    pub fn store_request(&mut self, id:Id, sender:oneshot::Sender<Reply>) {
        info!(self.logger,"Storing a new request {id}");
        // There will be no previous request, since Ids are assumed to be unique.
        // Still, if there was, we can just safely drop it.
        let _ = self.ongoing_calls.insert(id,sender);
    }

    pub fn clear(&mut self) {
        info!(self.logger,"Clearing all the requests.");
        self.ongoing_calls.clear()
    }

    pub fn complete_request(&mut self, id:Id, reply:Reply) -> FallibleResult<()> {
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

pub enum Disposition<Id,Reply,Notification> {
    Ignore,
    HandleReply {id:Id, reply:Reply},
    EmitEvent {event:Event<Notification>},
}

impl<Id,Reply,Notification> Disposition<Id,Reply,Notification> {
    pub fn notify(notification:Notification) -> Self {
        Disposition::EmitEvent {event:Event::Notification(notification)}
    }

    pub fn error(error:impl Into<failure::Error> + Debug) -> Self {
        Disposition::EmitEvent {event:Event::Error(error.into())}
    }
}

pub trait MessageProcessor<Id,Reply,Notification> = FnMut(TransportEvent) -> Disposition<Id,Reply,Notification>;

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
struct HandlerState<Id,Reply,Notification>
where Id:Eq+Hash+Debug,
      Notification:Debug,
      Reply:Debug, {
    #[derivative(Debug="ignore")]
    transport     : Box<dyn Transport>,
    logger        : Logger,
    sender        : Option<UnboundedSender<Event<Notification>>>,
    ongoing_calls : RequestHandler<Id,Reply>,
    #[derivative(Debug="ignore")]
    processor     : Box<dyn FnMut(TransportEvent) -> Disposition<Id,Reply,Notification>>,
}

impl <Id,Reply,Notification> HandlerState<Id,Reply,Notification>
    where Id: Copy + Debug + Display + Hash + Eq + Send + Sync + 'static,
          Notification:Debug,
          Reply:Debug, {
    fn new<T,P>(transport:T, logger:&Logger, processor:P) -> HandlerState<Id,Reply,Notification>
    where T : Transport + 'static,
          P : FnMut(TransportEvent) -> Disposition<Id,Reply,Notification> + 'static {
        HandlerState {
            transport : Box::new(transport),
            logger : logger.clone_ref(),
            sender : None,
            ongoing_calls : RequestHandler::new(logger),
            processor : Box::new(processor),
        }
    }

    fn emit_event(&mut self, event:Event<Notification>) {
        if let Some(mut sender) = self.sender.as_ref() {
            sender.send(event);
        }
    }

    fn process_reply(&mut self, id:Id, reply:Reply) {
        info!(self.logger,"Processing reply to request {id}");
        if let Err(error) = self.ongoing_calls.complete_request(id,reply) {
            self.emit_error(error);
        }
    }

    fn emit_notification(&mut self, notification:Notification) {
        info!(self.logger,"Emitting notification: {notification:?}");
        let event = Event::Notification(notification);
        self.emit_event(event);
    }

    fn emit_error(&mut self, error:impl Into<failure::Error> + Debug) {
        info!(self.logger,"Emitting error: {error:?}");
        let event = Event::Error(error.into());
        self.emit_event(event);
    }

    fn open_request(&mut self, id:Id, completer:oneshot::Sender<Reply>) {
        info!(self.logger,"Opening request: {id}");
        self.ongoing_calls.store_request(id, completer)
    }

    pub fn process_event(&mut self, event:TransportEvent) {
        info!(self.logger, "Transport event received: {event:?}");
        match (self.processor)(event) {
            Disposition::HandleReply {id,reply} => self.process_reply(id,reply),
            Disposition::EmitEvent {event} => self.emit_event(event),
            Disposition::Ignore => {}
        }
    }
}

#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct HandlerHandle<Id,Reply,Notification:Debug>
where Id : Eq+Hash+Debug,
      Notification:Debug,
      Reply:Debug, {
    logger : Logger,
    state  : Rc<RefCell<HandlerState<Id,Reply,Notification>>>,
}


pub trait MessageToServer : Debug {
    type Id : Copy;
    fn send(&self, transport:&mut dyn Transport) -> FallibleResult<()>;
    fn id(&self) -> Self::Id;
}

pub trait Decoder<Reply,R> = FnOnce(Reply) -> FallibleResult<R>;

impl <Id,Reply,Notification> HandlerHandle<Id,Reply,Notification>
where Id: Copy + Debug + Display + Hash + Eq + Send + Sync + 'static,
      Notification:Debug,
      Reply : Debug {

    pub fn new<T,P>(transport:T, logger:Logger, processor:P) -> Self
        where T : Transport + 'static,
              P : FnMut(TransportEvent) -> Disposition<Id,Reply,Notification> + 'static {
        let state = Rc::new(RefCell::new(HandlerState::new(transport,&logger,processor)));
        HandlerHandle {logger,state}
    }

    pub fn with_transport<R>(&self, f:impl FnOnce(&mut dyn Transport) -> R) -> R {
        f(self.state.borrow_mut().transport.deref_mut())
    }
    pub fn store_request(&self, id:Id, completer:oneshot::Sender<Reply>) {
        self.state.borrow_mut().open_request(id,completer)
    }

    pub fn open<F,R>(&self, message:&dyn MessageToServer<Id=Id>, f:F) -> impl Future<Output=FallibleResult<R>> + 'static
    where F: FnOnce(Reply) -> FallibleResult<R> + 'static,
          Reply: 'static {
        let id = message.id();

        info!(self.logger,"Sending message {message:?}");
        self.with_transport(|transport| message.send(transport));

        let (sender, receiver) = oneshot::channel::<Reply>();
        let logger = self.logger.clone_ref();
        let ret = receiver.map(move |result_or_cancel| {
            info!(logger, "Processing request reply {result_or_cancel:?}");
            let result = result_or_cancel?;
            f(result)
        });

        info!(self.logger,"Opening request: {id}");
        self.store_request(id,sender);
        ret
    }

    pub fn runner(&self) -> impl Future<Output = ()> {
        let event_receiver = self.with_transport(|t| t.establish_event_stream());

        let state = Rc::downgrade(&self.state);
        event_receiver.for_each(move |event: TransportEvent| {
            if let Some(state) = state.upgrade() {
                state.borrow_mut().process_event(event);
            }
            futures::future::ready(())
        })
    }
}
