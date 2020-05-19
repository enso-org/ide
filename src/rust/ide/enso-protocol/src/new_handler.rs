use crate::prelude::*;
use json_rpc::{Transport, TransportEvent};
use futures::channel::mpsc::UnboundedSender;
//use futures::channel::oneshot;
//use futures::channel::oneshot::Canceled;
//use futures::SinkExt;

use logger::*;
use std::future::Future;
use utils::fail::FallibleResult;
//use json_rpc::error::RpcError;
use crate::common::ongoing_calls::OngoingCalls;
use crate::common::event::Event;

/// Describes how the given server's message should be dealt with.
#[derive(Debug)]
pub enum Disposition<Id,Reply,Notification>
where Id:Debug, Reply:Debug, Notification:Debug {
    /// Ignore the message.
    Ignore,
    /// Treat as a reply to an open request.
    HandleReply {
        /// Remote Call ID (correlation ID).
        id:Id,
        /// The reply contents.
        reply:Reply
    },
    /// Emit given event (usually error or a notification).
    EmitEvent {
        /// Event to be emitted.
        event:Event<Notification>
    },
}

impl<Id,Reply,Notification> Disposition<Id,Reply,Notification>
where Id:Debug, Reply:Debug, Notification:Debug {
    pub fn notify(notification:Notification) -> Self {
        Disposition::EmitEvent {event:Event::Notification(notification)}
    }

    pub fn error(error:impl Into<failure::Error> + Debug) -> Self {
        Disposition::EmitEvent {event:Event::Error(error.into())}
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
struct HandlerData<Id,Reply,Notification>
where Id           : Eq+Hash+Debug,
      Notification : Debug,
      Reply        : Debug, {
    #[derivative(Debug="ignore")]
    transport     : Box<dyn Transport>,
    logger        : Logger,
    sender        : Option<UnboundedSender<Event<Notification>>>,
    ongoing_calls : OngoingCalls<Id,Reply>,
    #[derivative(Debug="ignore")]
    processor     : Box<dyn FnMut(TransportEvent) -> Disposition<Id,Reply,Notification>>,
}

impl <Id,Reply,Notification> HandlerData<Id,Reply,Notification>
where Id           : Copy + Debug + Display + Hash + Eq + Send + Sync + 'static,
      Notification : Debug,
      Reply        : Debug, {
    fn new<T,P>(transport:T, logger:&Logger, processor:P) -> HandlerData<Id,Reply,Notification>
    where T : Transport + 'static,
          P : FnMut(TransportEvent) -> Disposition<Id,Reply,Notification> + 'static {
        HandlerData {
            transport     : Box::new(transport),
            logger        : logger.clone_ref(),
            sender        : None,
            ongoing_calls : OngoingCalls::new(logger),
            processor     : Box::new(processor),
        }
    }

    fn emit_event(&mut self, event:Event<Notification>) {
        if let Some(sender) = self.sender.as_ref() {
            // Error can happen if there is no listener. But we don't mind this.
            let _ = sender.unbounded_send(event);
        }
    }

    /// Feeds the reply to complete the corresponding open request.
    fn process_reply(&mut self, id:Id, reply:Reply) {
        info!(self.logger,"Processing reply to request {id}: {reply:?}");
        if let Err(error) = self.ongoing_calls.complete_request(id,reply) {
            self.emit_error(error);
        }
    }

    /// Helper that wraps error into an appropriate event value and emits it.
    fn emit_error(&mut self, error:impl Into<failure::Error> + Debug) {
        info!(self.logger,"Emitting error: {error:?}");
        let event = Event::Error(error.into());
        self.emit_event(event);
    }

    /// Handles incoming transport event. The `processor` is used to decide the further processing
    /// path.
    ///
    /// Main entry point for input data while running. Should be connected to the `Transport`s
    /// output event stream.
    pub fn process_event(&mut self, event:TransportEvent) {
        info!(self.logger, "Transport event received: {event:?}");
        match (self.processor)(event) {
            Disposition::HandleReply {id,reply} => self.process_reply(id,reply),
            Disposition::EmitEvent {event} => self.emit_event(event),
            Disposition::Ignore => {}
        }
    }

    pub fn make_request<F,R>
    (&mut self, message:&dyn Request<Id=Id>, f:F) -> impl Future<Output=FallibleResult<R>>
    where F: FnOnce(Reply) -> FallibleResult<R> {
        let id  = message.id();
        let ret = self.ongoing_calls.open_new_request(id,f);
        info!(self.logger,"Sending message {message:?}");
        let sending_result = message.send(self.transport.as_mut());
        if sending_result.is_err() {
            // If we failed to send the request, it should be immediately removed.
            // This will result in the returned future immediately yielding error.
            self.ongoing_calls.remove_request(&id);
        }
        ret
    }
}

#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Handler<Id,Reply,Notification:Debug>
where Id           : Eq+Hash+Debug,
      Notification : Debug,
      Reply        : Debug, {
    logger : Logger,
    state  : Rc<RefCell<HandlerData<Id,Reply,Notification>>>,
}

/// A value that can be used to represent a request to remote RPC server.
pub trait Request : Debug {
    /// Request ID.
    type Id : Copy;

    /// Send the message to the peer using the provided transport.
    fn send(&self, transport:&mut dyn Transport) -> FallibleResult<()>;

    /// Request ID, that will be used later to associate peer's response.
    fn id(&self) -> Self::Id;
}

impl <Id,Reply,Notification> Handler<Id,Reply,Notification>
where Id           : Copy + Debug + Display + Hash + Eq + Send + Sync + 'static,
      Notification : Debug,
      Reply        : Debug {

    /// Creates a new handler operating over given transport.
    ///
    /// `processor` must deal with decoding incoming transport events.
    pub fn new<T,P>(transport:T, logger:Logger, processor:P) -> Self
    where T : Transport + 'static,
          P : FnMut(TransportEvent) -> Disposition<Id,Reply,Notification> + 'static {
        let state = Rc::new(RefCell::new(HandlerData::new(transport,&logger,processor)));
        Handler {logger,state}
    }

    /// Starts a new request described by a given message.
    ///
    /// The request shall be sent to the server and then await the reply.
    pub fn make_request<F,R>
    (&self, message:&dyn Request<Id=Id>, f:F) -> impl Future<Output=FallibleResult<R>>
    where F: FnOnce(Reply) -> FallibleResult<R> {
        self.state.borrow_mut().make_request(message,f)
    }

    /// See the `runner` on the `Client`.
    pub fn runner(&self) -> impl Future<Output = ()> {
        let event_receiver = self.state.borrow_mut().transport.establish_event_stream();
        let state = Rc::downgrade(&self.state);
        event_receiver.for_each(move |event: TransportEvent| {
            if let Some(state) = state.upgrade() {
                state.borrow_mut().process_event(event);
            }
            futures::future::ready(())
        })
    }
}
