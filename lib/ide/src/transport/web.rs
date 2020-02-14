//! web_sys::WebSocket-based `Transport` implementation.

use crate::prelude::*;

use failure::Error;
use js_sys::Function;
use json_rpc::Transport;
use json_rpc::TransportEvent;
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::JsCast;
use web_sys::CloseEvent;
use web_sys::Event;
use web_sys::MessageEvent;




// =================
// === Utilities ===
// =================

#[wasm_bindgen(inline_js = "export function stringify(a) { return a + ''; }")]
extern "C" {
    /// Convert given JS value into a string.
    fn stringify(a: JsValue) -> String;
}



// ==============
// === Errors ===
// ==============


// === ConnectingError ===

/// Errors that may happen when trying to establish WebSocket connection.
#[derive(Debug)]
#[derive(Fail)]
pub enum ConnectingError {
    /// Failed to construct websocket. Usually this happens due to bad URL.
    #[fail(display = "Invalid websocket specification: {}.", _0)]
    ConstructionError(String),
    /// Failed to establish connection. Usually due to connectivity issues,
    /// wrong URL or server being down.
    #[fail(display = "Failed to establish connection.")]
    FailedToConnect,
}


// === ConnectingError ===

/// Error that may occur when attempting to send the data ove WSTransport.
#[derive(Debug, Fail)]
enum SendingError {
    #[fail(display = "Failed to send message. Exception: {:?}.", _0)]
    FailedToSend(String),

    #[fail(display = "Failed to send message because socket state is {:?}.", _0)]
    NotOpen(State),
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
    /// See: https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/readyState
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



// ======================
// === ClosureStorage ===
// ======================

/// Constraint for JS closure argument types
pub trait ClosureArg = FromWasmAbi + 'static;

/// Stores an optional closure.
#[derive(Debug,Derivative)]
#[derivative(Default(bound=""))]
pub struct ClosureStorage<Arg> {
    pub closure : Option<Closure<dyn FnMut(Arg)>>,
}

impl <Arg> ClosureStorage<Arg> {
    /// An empty closure storage.
    pub fn new() -> ClosureStorage<Arg> {
        default()
    }

    /// Stores the given closure.
    pub fn store(&mut self, closure:Closure<dyn FnMut(Arg)>) {
        self.closure = Some(closure);
    }

    /// Obtain JS reference to the closure (that can be passed e.g. as a callback
    /// to an event handler).
    pub fn js_ref(&self) -> Option<&Function> {
        self.closure.as_ref().map(|closure| closure.as_ref().unchecked_ref() )
    }

    /// Wraps given function into a Closure.
    pub fn wrap(&mut self, f:impl FnMut(Arg) + 'static)
    where Arg: ClosureArg {
        let boxed = Box::new(f);
        let wrapped = Closure::wrap(boxed);
        self.store(wrapped);
    }

    /// Clears the current closure.
    /// Note: if reference to it is still used by JS, it will throw an exception
    /// on calling attempt.
    pub fn clear(&mut self) {
        self.closure = None;
    }
}



// =================
// === WebSocket ===
// =================

#[derive(Debug)]
pub struct WebSocket {
    /// Handle to the JS `WebSocket` object.
    pub ws         : web_sys::WebSocket,
    /// Handle to a closure connected to `WebSocket.onmessage`.
    pub on_message : ClosureStorage<MessageEvent>,
    /// Handle to a closure connected to `WebSocket.onclose`.
    pub on_close   : ClosureStorage<CloseEvent>,
    /// Handle to a closure connected to `WebSocket.onopen`.
    pub on_open    : ClosureStorage<Event>,
    /// Handle to a closure connected to `WebSocket.onerror`.
    pub on_error   : ClosureStorage<Event>,
}

impl WebSocket {
    /// Wraps given WebSocket object.
    pub fn new(ws:web_sys::WebSocket) -> WebSocket {
        WebSocket {
            ws,
            on_message : default(),
            on_close   : default(),
            on_open    : default(),
            on_error   : default(),
        }
    }

    /// Establish connection with endpoint defined by the given URL and wrap it.
    pub async fn new_connected(url:impl Str) -> Result<WebSocket,ConnectingError> {
        let     ws  = web_sys::WebSocket::new(url.as_ref());
        let mut wst = WebSocket::new(ws.map_err(|e| {
            ConnectingError::ConstructionError(stringify(e))
        })?);

        // Connecting attempt shall either emit on_open or on_close.
        // We shall wait for whatever comes first.
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<Result<(),()>>();
        let tx_clone = tx.clone();
        wst.set_on_close(move |_| {
            tx_clone.unbounded_send(Err(())).ok();
        });

        let tx_clone = tx.clone();
        wst.set_on_open(move |_| {
            tx_clone.unbounded_send(Ok(())).ok();
        });

        match rx.next().await {
            Some(Ok(())) => {
                wst.clear_callbacka();
                Ok(wst)
            }
            _ => Err(ConnectingError::FailedToConnect)
        }
    }

    /// Checks the current state of the connection.
    pub fn state(&self) -> State {
        State::query_ws(&self.ws)
    }

    /// Sets callback for `close` event.
    pub fn set_on_close(&mut self, f:impl FnMut(CloseEvent) + 'static) {
        self.on_close.wrap(f);
        self.ws.set_onclose(self.on_close.js_ref());
    }

    /// Sets callback for `error` event.
    pub fn set_on_error(&mut self, f:impl FnMut(Event) + 'static) {
        self.on_error.wrap(f);
        self.ws.set_onerror(self.on_error.js_ref());
    }

    /// Sets callback for `message` event.
    pub fn set_on_message(&mut self, f:impl FnMut(MessageEvent) + 'static) {
        self.on_message.wrap(f);
        self.ws.set_onmessage(self.on_message.js_ref());
    }

    /// Sets callback for `open` event.
    pub fn set_on_open(&mut self, f:impl FnMut(Event) + 'static) {
        self.on_open.wrap(f);
        self.ws.set_onopen(self.on_open.js_ref());
    }

    /// Clears all callbacks.
    pub fn clear_callbacka(&mut self) {
        self.on_close  .clear();
        self.on_error  .clear();
        self.on_message.clear();
        self.on_open   .clear();
        self.ws.set_onclose(None);
        self.ws.set_onerror(None);
        self.ws.set_onmessage(None);
        self.ws.set_onopen(None);
    }
}

impl Transport for WebSocket {
    fn send_text(&mut self, message:String) -> Result<(), Error> {
        // Sending through the closed WebSocket can return Ok() with error only
        // appearing in the log. We explicitly check for this to get failure as
        // early as possible.
        //
        // If WebSocket closes after the check, caller will be able to handle it
        // when receiving `TransportEvent::Closed`.
        let state = self.state();
        if state != State::Open {
            return Err(SendingError::NotOpen(state))?;
        }

        self.ws.send_with_str(&message).map_err(|e| {
            SendingError::FailedToSend(stringify(e)).into()
        })
    }

    fn set_event_tx(&mut self, tx:futures::channel::mpsc::UnboundedSender<TransportEvent>) {
        let tx1 = tx.clone();
        self.set_on_message(move |e| {
            let data = e.data();
            if let Some(text) = data.as_string() {
                let _ = tx1.unbounded_send(TransportEvent::TextMessage(text));
            }
        });

        let tx2 = tx.clone();
        self.set_on_close(move |_e| {
            let _ = tx2.unbounded_send(TransportEvent::Closed);
        });

        self.set_on_open(move |_e| {
            let _ = tx.unbounded_send(TransportEvent::Opened);
        });
    }
}
