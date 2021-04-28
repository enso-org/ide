//! web_sys::WebSocket-based `Transport` implementation.

use crate::prelude::*;

use ensogl_system_web::closure::storage::OptionalFmMutClosure;
use ensogl_system_web::js_to_string;
use failure::Error;
use futures::channel::mpsc;
use json_rpc::Transport;
use json_rpc::TransportEvent;
use utils::channel;
use wasm_bindgen::JsCast;
use web_sys::BinaryType;
use web_sys::CloseEvent;
use js_sys::Function;
use web_sys::Event;
use web_sys::MessageEvent;
use ensogl::system::web::closure::storage::ClosureFn;
use web_sys::EventTarget;

use enso_logger::DefaultTraceLogger as Logger;
use futures::TryFutureExt;


// ==============
// === Errors ===
// ==============

/// Errors that may happen when trying to establish WebSocket connection.
#[derive(Clone,Debug,Fail)]
pub enum ConnectingError {
    /// Failed to construct websocket. Usually this happens due to bad URL.
    #[fail(display = "Invalid websocket specification: {}.", _0)]
    ConstructionError(String),
    /// Failed to establish connection. Usually due to connectivity issues,
    /// wrong URL or server being down. Unfortunately, while the real error
    /// cause is usually logged down in js console, we have no reliable means of
    /// obtaining it programmatically. Reported error codes are utterly
    /// unreliable.
    #[fail(display = "Failed to establish connection.")]
    FailedToConnect,
}

/// Error that may occur when attempting to send the data over WebSocket
/// transport.
#[derive(Clone,Debug,Fail)]
pub enum SendingError {
    /// Calling `send` method has resulted in an JS exception.
    #[fail(display = "Failed to send message. Exception: {:?}.", _0)]
    FailedToSend(String),
    /// The socket was already closed, even before attempting sending a message.
    #[fail(display = "Failed to send message because socket state is {:?}.", _0)]
    NotOpen(State),
}

impl SendingError {
    /// Constructs from the error yielded by one of the JS's WebSocket sending functions.
    pub fn from_send_error(error:JsValue) -> SendingError {
        SendingError::FailedToSend(js_to_string(&error))
    }
}



// =============
// === State ===
// =============

/// Describes the current state of WebSocket connection.
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum State {
    /// Socket has been created. The connection is not yet open.
    Connecting,
    /// The connection is open and ready to communicate.
    Open,
    /// The connection is in the process of closing.
    Closing,
    /// The connection is closed or couldn't be opened.
    Closed,
    /// Any other, unknown condition.
    Unknown(u16),
}

impl State {
    /// Returns current state of the given WebSocket.
    pub fn query_ws(ws:&web_sys::WebSocket) -> State {
        State::from_code(ws.ready_state())
    }

    /// Translates code returned by `WebSocket.readyState` into our enum.
    /// cf https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/readyState
    pub fn from_code(code:u16) -> State {
        match code {
            web_sys::WebSocket::CONNECTING => State::Connecting,
            web_sys::WebSocket::OPEN       => State::Open,
            web_sys::WebSocket::CLOSING    => State::Closing,
            web_sys::WebSocket::CLOSED     => State::Closed,
            num                            => State::Unknown(num), // impossible
        }
    }
}

pub trait JsEvent {
    type Arg;
    const NAME:&'static str;
    fn add_listener(target:impl AsRef<EventTarget>, listener:&Function) -> Result<(),JsValue> {
        target.as_ref().add_event_listener_with_callback(Self::NAME,listener)
    }

    fn remove_listener(target:impl AsRef<EventTarget>, listener:&Function) -> Result<(),JsValue> {
        target.as_ref().remove_event_listener_with_callback(Self::NAME,listener)
    }
}

#[derive(Debug,Default)]
pub struct OnOpen;
impl JsEvent for OnOpen {
    type Arg = Event;
    const NAME:&'static str = "open";
}

#[derive(Debug,Default)]
pub struct OnClose;
impl JsEvent for OnClose {
    type Arg = CloseEvent;
    const NAME:&'static str = "close";
}

#[derive(Debug,Default)]
pub struct OnMessage;
impl JsEvent for OnMessage {
    type Arg = MessageEvent;
    const NAME:&'static str = "message";
}

#[derive(Debug,Default)]
pub struct OnError;
impl JsEvent for OnError {
    type Arg = Event;
    const NAME:&'static str = "error";
}

#[derive(Derivative)]
#[derivative(Debug(bound="Event::Arg: Debug"))]
pub struct EventListener<Event:JsEvent> {
    logger     : Logger,
    #[derivative(Debug="ignore")]
    target     : Option<Box<dyn AsRef<EventTarget>>>,
    js_closure : OptionalFmMutClosure<Event::Arg>,
}

impl<Event:JsEvent> Default for EventListener<Event> {
    fn default() -> Self {
        Self {
            logger:Logger::new(Event::NAME),
            target:default(),
            js_closure:default(),
        }
    }
}

impl<Event: JsEvent> EventListener<Event> {
    pub fn new(logger:impl AnyLogger) -> Self where Self:Default {
        Self {
            logger : Logger::sub(logger,Event::NAME),
            ..default()
        }
    }

    pub fn target(&self) -> Result<&EventTarget,JsValue> {
        let err = || js_sys::Error::new("No target object provided.");
        self.target.as_ref().ok_or_else(err).map(AsRef::as_ref).map(AsRef::as_ref).map_err(Into::into)
    }

    pub fn js_function(&self) -> Result<&Function,JsValue> {
        let err = || js_sys::Error::new("No closure has been set.");
        self.js_closure.js_ref().ok_or_else(err).map_err(Into::into)
    }

    fn add_if_active(&mut self) -> Result<(),JsValue> {
        match (self.target(), self.js_function()) {
            (Ok(target),Ok(function)) => {
                let name = Event::NAME;
                info!(self.logger,"Attaching callback to event {name} on {js_to_string(target)}");
                Event::add_listener(target,function)
            },
            _ => Ok(()),
        }
    }

    fn remove_if_active(&mut self) -> Result<(),JsValue> {
        match (self.target(), self.js_function()) {
            (Ok(target),Ok(function)) => {
                let name = Event::NAME;
                info!(self.logger,"Detaching callback to event {name} on {js_to_string(target)}");
                Event::remove_listener(target, function)
            },
            _ => Ok(()),
        }
    }

    // fn remove_internal(&mut self, target:impl AsRef<EventTarget>, function:&Function) -> Result<(),JsValue> {
    //     Event::remove_listener(target,function)?;
    //     self.target = None;
    //     Ok(())
    // }
    //
    // fn remove_if_active(&mut self) -> Result<(),JsValue> {
    //     match (self.target(), self.js_function()) {
    //         (Ok(target),Ok(function)) => self.remove_internal(target,function),
    //         _                         => Ok(()),
    //     }
    // }

    pub fn set_target(&mut self, target:&dyn AsRef<EventTarget>) -> Result<(),JsValue> {
        self.remove_if_active()?;
        self.target = Some(Box::new(target.as_ref().clone()));
        self.add_if_active()
    }

    pub fn set_callback(&mut self, f:impl ClosureFn<Event::Arg>) -> Result<(),JsValue> {
        self.remove_if_active()?;
        self.js_closure.wrap(f);
        self.add_if_active()
    }

    pub fn clear_callback(&mut self) -> Result<(),JsValue> {
        self.remove_if_active()?;
        self.js_closure.clear();
        Ok(())
    }

    // pub fn detach_from_target(&mut self, target:&dyn AsRef<EventTarget>) -> Result<(),JsValue> {
    //     self.remove_if_active()?;
    //     self.target = Some(Box::new(target.as_ref().clone()));
    //     if let Ok(function) = self.js_function() {
    //         Event::add_listener(&self.target,function)?;
    //     }
    //     Ok(())
    // }
    //
    // pub fn remove(&mut self) -> Result<(),JsValue> {
    //     self.remove_internal(self.target()?, self.js_function()?)
    // }
}


/// Wrapper over JS `WebSocket` object and callbacks to its signals.
#[derive(Debug)]
pub struct Model {
    #[allow(missing_docs)]
    pub logger    : Logger,
    /// Handle to the JS `WebSocket` object.
    pub ws        : web_sys::WebSocket,
    pub on_close: EventListener<OnClose>,
    pub on_message: EventListener<OnMessage>,
    pub on_open: EventListener<OnOpen>,
    pub on_error: EventListener<OnError>,
}

impl Model {
    /// Wraps given WebSocket object.
    pub fn new(ws:web_sys::WebSocket, logger:Logger) -> Model {
        ws.set_binary_type(BinaryType::Arraybuffer);
        let mut ret = Model {
            ws,
            on_close: EventListener::new(&logger),
            on_message: EventListener::new(&logger),
            on_open: EventListener::new(&logger),
            on_error: EventListener::new(&logger),
            logger,
        };

        ret.on_close.set_target(&ret.ws);
        ret.on_message.set_target(&ret.ws);
        ret.on_open.set_target(&ret.ws);
        ret.on_error.set_target(&ret.ws);
        ret
    }

    pub fn close(&mut self, reason:&str) -> Result<(),JsValue> {
        self.ws.close_with_code_and_reason(1000,reason)?;
        self.clear_callbacks()
    }

    /// Clears all the available callbacks.
    pub fn clear_callbacks(&mut self) -> Result<(),JsValue> {
        let Self{ws,on_close,on_error,on_message,on_open,logger:_logger} = self;
        // We don't care if removing actually removed anything.
        // If callbacks were not set, then they are clear from the start.
        on_close.clear_callback()?;
        on_error.clear_callback()?;
        on_message.clear_callback()?;
        on_open.clear_callback()
    }

    pub fn reconnect(&mut self) -> Result<(),JsValue> {
        let url = self.ws.url();
        warning!(self.logger, "Reconnecting WS to {url}");

        let new_ws = web_sys::WebSocket::new(&url)?;

        self.on_close.  set_target(&new_ws)?;
        self.on_error.  set_target(&new_ws)?;
        self.on_message.set_target(&new_ws)?;
        self.on_open.   set_target(&new_ws)?;
        self.ws = new_ws;

        Ok(())
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        if let Err(e) = self.close("Rust Value has been dropped.") {
            error!(self.logger,"Error when closing socket due to being dropped: {js_to_string(&e)}")
        }
    }
}

// =================
// === WebSocket ===
// =================

/// Wrapper over JS `WebSocket` object and callbacks to its signals.
#[derive(Debug)]
pub struct WebSocket {
    #[allow(missing_docs)]
    pub logger     : DefaultTraceLogger,
    model : Rc<RefCell<Model>>,
}

impl WebSocket {
    /// Wraps given WebSocket object.
    pub fn new
    (ws:web_sys::WebSocket, parent:impl AnyLogger, name:impl AsRef<str>) -> WebSocket {
        let logger = DefaultTraceLogger::sub(parent,name);
        let model = Model::new(ws,logger.clone());

        WebSocket {
            logger,
            model  : Rc::new(RefCell::new(model)),
        }
    }

    /// Establish connection with endpoint defined by the given URL and wrap it.
    /// Asynchronous, because it waits until connection is established.
    pub async fn new_opened
    (parent:impl AnyLogger, url:impl Str) -> Result<WebSocket,ConnectingError> {
        let ws = web_sys::WebSocket::new(url.as_ref()).map_err(|e| {
            ConnectingError::ConstructionError(js_to_string(&e))
        })?;
        let mut wst = WebSocket::new(ws,&parent,url.into());
        wst.wait_until_open().await?;
        Ok(wst)
    }

    /// Awaits until `open` signal has been emitted. Clears any callbacks on
    /// this `WebSocket`, if any has been set.
    async fn wait_until_open(&mut self) -> Result<(),ConnectingError> {
        // Connecting attempt shall either emit on_open or on_close.
        // We shall wait for whatever comes first.
        let (transmitter, mut receiver) = mpsc::unbounded::<Result<(),()>>();
        let transmitter_clone = transmitter.clone();

        self.set_on_close(move |_| {
            // Note [mwu] Ignore argument, `CloseEvent` here contains rubbish
            // anyway, nothing useful to pass to caller. Error code or reason
            // string should not be relied upon.
            utils::channel::emit(&transmitter_clone, Err(()));
        });
        self.set_on_open(move |_| {
            utils::channel::emit(&transmitter, Ok(()));
        });

        match receiver.next().await {
            Some(Ok(())) => {
                self.model.borrow_mut().clear_callbacks();
                info!(self.logger, "Connection opened.");
                Ok(())
            }
            _ => Err(ConnectingError::FailedToConnect)
        }
    }

    /// Checks the current state of the connection.
    pub fn state(&self) -> State {
        State::query_ws(&self.model.borrow().ws)
    }

    fn with_mut_model<R>(&mut self, f:impl FnOnce(&mut Model) -> R) -> R {
        with(self.model.borrow_mut(), |mut model| f(model.deref_mut()))
    }

    /// Sets callback for the `close` event.
    pub fn set_on_close(&mut self, mut f:impl FnMut(CloseEvent) + 'static) -> Result<(),JsValue> {
        let model = Rc::downgrade(&self.model);
        // We add our own layer on top of given callback to reconnect on closing.
        // JS allows attaching only one callback for event, so they must be together.
        let f = move |event| {
            // First we call callback. Perhaps in reaction to closing it will delete the model
            // and the reconnecting won't be needed.
            f(event);
            if let Some(model) = model.upgrade() {
                model.borrow_mut().reconnect();
            }
        };
        self.with_mut_model(move |model| model.on_close.set_callback(f))
    }

    /// Sets callback for the `error` event.
    pub fn set_on_error(&mut self, f:impl FnMut(Event) + 'static) -> Result<(),JsValue> {
        self.with_mut_model(move |model| model.on_error.set_callback(f))
    }

    /// Sets callback for the `message` event.
    pub fn set_on_message(&mut self, f:impl FnMut(MessageEvent) + 'static) -> Result<(),JsValue> {
        self.with_mut_model(move |model| model.on_message.set_callback(f))
    }

    /// Sets callback for the `open` event.
    pub fn set_on_open(&mut self, f:impl FnMut(Event) + 'static) -> Result<(),JsValue> {
        self.with_mut_model(move |model| model.on_open.set_callback(f))
    }

    /// Executes a given function with a mutable reference to the socket.
    /// The function should attempt sending the message through the websocket.
    ///
    /// Fails if the socket is not opened or if the sending function failed.
    /// The error from `F` shall be translated into `SendingError`.
    ///
    /// WARNING: `f` works under borrow_mut and must not give away control.
    fn send_with_open_socket<F,R>(&mut self, f:F) -> Result<R,Error>
    where F : FnOnce(&mut web_sys::WebSocket) -> Result<R,JsValue> {
        // Sending through the closed WebSocket can return Ok() with error only
        // appearing in the log. We explicitly check for this to get failure as
        // early as possible.
        //
        // If WebSocket closes after the check, caller will be able to handle it
        // when receiving `TransportEvent::Closed`.
        let state = self.state();
        if state != State::Open {
            Err(SendingError::NotOpen(state).into())
        } else {
            let result = f(&mut self.model.borrow_mut().ws);
            result.map_err(|e| SendingError::from_send_error(e).into())
        }
    }
}

impl Transport for WebSocket {
    fn send_text(&mut self, message:&str) -> Result<(), Error> {
        info!(self.logger, "Sending text message of length {message.len()}");
        debug!(self.logger, "Message contents: {message}");
        self.send_with_open_socket(|ws| ws.send_with_str(message))
    }

    fn send_binary(&mut self, message:&[u8]) -> Result<(), Error> {
        info!(self.logger, "Sending binary message of length {message.len()}");
        debug!(self.logger,|| format!("Message contents: {:x?}", message));
        // TODO [mwu]
        //   Here we workaround issue from wasm-bindgen 0.2.58:
        //   https://github.com/rustwasm/wasm-bindgen/issues/2014
        //   The issue has been fixed in 0.2.59, however we can't upgrade, as we rely on fragile
        //   regexp-based machinery to process wasm-bindgen output.
        //   When fixed, we should pass `message` directly, without intermediate copy.
        let mut owned_copy = Vec::from(message);
        let mut_slice      = owned_copy.as_mut();
        self.send_with_open_socket(|ws| ws.send_with_u8_array(mut_slice))
    }

    fn set_event_transmitter(&mut self, transmitter:mpsc::UnboundedSender<TransportEvent>) {
        info!(self.logger,"Setting event transmitter.");
        let transmitter_copy = transmitter.clone();
        let logger_copy = self.logger.clone_ref();
        self.set_on_message(move |e| {
            let data = e.data();
            if let Some(text) = data.as_string() {
                debug!(logger_copy, "Received a text message: {text}");
                channel::emit(&transmitter_copy,TransportEvent::TextMessage(text));
            } else if let Ok(array_buffer) = data.dyn_into::<js_sys::ArrayBuffer>() {
                let array       = js_sys::Uint8Array::new(&array_buffer);
                let binary_data = array.to_vec();
                debug!(logger_copy,|| format!("Received a binary message: {:x?}", binary_data));
                let event = TransportEvent::BinaryMessage(binary_data);
                channel::emit(&transmitter_copy,event);
            } else {
                info!(logger_copy,"Received other kind of message: {js_to_string(&e.data())}.");
            }
        });

        let transmitter_copy = transmitter.clone();
        let logger_copy = self.logger.clone_ref();
        self.set_on_close(move |_e| {
            info!(logger_copy,"Connection has been closed.");
            channel::emit(&transmitter_copy,TransportEvent::Closed);
        });

        let logger_copy = self.logger.clone_ref();
        self.set_on_open(move |_e| {
            info!(logger_copy,"Connection has been opened.");
            channel::emit(&transmitter, TransportEvent::Opened);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use utils::test::traits::*;
    use ensogl::system::web;
    use crate::executor::test_utils::TestWithLocalPoolExecutor;
    use std::time::Duration;
    use ensogl_system_web::sleep;


    #[wasm_bindgen_test::wasm_bindgen_test(async)]
    #[allow(dead_code)]
    async fn websocket_fun() {
        let logger = DefaultTraceLogger::new("Test");

        web::set_stdout();
        println!("Start");

        let executor = executor::web::EventLoopExecutor::new_running();
        let _executor = crate::initializer::setup_global_executor();
        executor::global::set_spawner(executor.spawner.clone());
        std::mem::forget(executor);

        let ws     = WebSocket::new_opened(logger,"ws://localhost:30445").await;
        let mut ws = ws.expect("Couldn't connect to WebSocket server.");
        println!("{:?}",ws);
        println!("Pre-Endut");

        let handler = ws.establish_event_stream().for_each(|arg| {
            println!("{:?}",arg);
            futures::future::ready(())
        });
        executor::global::spawn(handler);


        web::sleep(Duration::from_secs(20)).await;
        println!("Endut");
    }
}
