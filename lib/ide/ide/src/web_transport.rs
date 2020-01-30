use crate::prelude::*;

use crate::log;

use js_sys::Function;
use json_rpc::Transport;
use json_rpc::TransportEvent;
use failure::Error;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::CloseEvent;
use web_sys::MessageEvent;
use web_sys::WebSocket;

/// Describes the current state of WebSocket connection.
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum WebSocketState {
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

impl WebSocketState {
    /// Returns current state of the given WebSocket.
    pub fn query_ws(ws:&web_sys::WebSocket) -> WebSocketState {
        WebSocketState::from_code(ws.ready_state())
    }

    /// Translates code returned by `WebSocket.readyState` into our enum.
    /// See: https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/readyState
    pub fn from_code(code:u16) -> WebSocketState {
        match code {
            WebSocket::CONNECTING => WebSocketState::Connecting,
            WebSocket::OPEN       => WebSocketState::Open,
            WebSocket::CLOSING    => WebSocketState::Closing,
            WebSocket::CLOSED     => WebSocketState::Closed,
            num                   => WebSocketState::Unknown(num), // impossible
        }
    }
}

#[derive(Debug)]
pub struct MyWebSocket {
    /// Handle to the JS `WebSocket` object.
    pub ws         : web_sys::WebSocket,
    /// Handle to a closure connected to `WebSocket.onmessage`.
    pub on_message : ClosureStorage<MessageEvent>,
    /// Handle to a closure connected to `WebSocket.onclose`.
    pub on_close   : ClosureStorage<CloseEvent>,
}

#[derive(Debug,Derivative)]
#[derivative(Default(bound=""))]
pub struct ClosureStorage<T> {
    pub closure:Option<Closure<dyn FnMut(T)>>,
}

impl <T> ClosureStorage<T> {
    pub fn new() -> ClosureStorage<T> {
        default()
    }
    pub fn store(&mut self, closure:Closure<dyn FnMut(T)>) {
        self.closure = Some(closure);
    }
    pub fn js_ref(&self) -> Option<&Function> {
        self.closure.as_ref().map(|closure| closure.as_ref().unchecked_ref() )
    }
}

impl MyWebSocket {
    pub async fn new(url:&str) -> MyWebSocket {
        MyWebSocket {
            ws         : new_websocket(url).await,
            on_message : default(),
            on_close   : default(),
        }
    }

    pub fn state(&self) -> WebSocketState {
        WebSocketState::query_ws(&self.ws)
    }
}

#[derive(Debug, Fail)]
enum SendingError {
    #[fail(display = "Failed to send message. Exception: {:?}.", _0)]
    FailedToSend(String),

    #[fail(display = "Failed to send message because socket state is {:?}.", _0)]
    NotOpen(WebSocketState),
}

impl Transport for MyWebSocket {
    fn send_text(&mut self, message:String) -> Result<(), Error> {
        log!("will send text message: {}", message);
        log!("ws declared state: {:?}", WebSocketState::query_ws(&self.ws));

        // Sending through the closed WebSocket can return Ok() with error only
        // appearing in the log. We explicitly check for this to get failure as
        // early as possible.
        //
        // If WebSocket closes after the check, caller will be able to handle it
        // when receiving `TransportEvent::Closed`.
        let state = self.state();
        if state != WebSocketState::Open {
            return Err(SendingError::NotOpen(state))?;
        }

        let ret = self.ws.send_with_str(&message);
        let ret = ret.map_err(|e| {
            SendingError::FailedToSend(format!("{:?}", e)).into()
        });
        log!("Sending result: {:?}", ret);
        ret
//        Ok(ret?)
    }

    fn set_event_tx(&mut self, tx:std::sync::mpsc::Sender<TransportEvent>) {
        let tx1 = tx.clone();
        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            let data = e.data();
            if let Some(text) = data.as_string() {
                let _ = tx1.send(TransportEvent::TextMessage(text));
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        self.on_message.store(on_message);
        self.ws.set_onmessage(self.on_message.js_ref());

        let on_close = Closure::wrap(Box::new(move |_e:CloseEvent| {
            let _ = tx.send(TransportEvent::Closed);
        }) as Box<dyn FnMut(CloseEvent)>);
        self.on_close.store(on_close);
        self.ws.set_onclose(self.on_close.js_ref());
    }
}

pub async fn new_websocket(url:&str) -> WebSocket {
    log!("Starting new WebSocket connecting to {}...", url);
    let (sender, receiver) = futures::channel::oneshot::channel::<WebSocket>();
    let ws = WebSocket::new(url).unwrap();
    let cloned_ws = ws.clone();
    let sender = Rc::new(RefCell::new(Some(sender)));
    let onopen_callback = Closure::wrap(Box::new(move |_| {
        if let Some(s) = sender.borrow_mut().take() {
            log!("WebSocket successfully opened!");
            let _ = s.send(cloned_ws.clone());
            cloned_ws.set_onopen(None);
        }
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();
    receiver.await.unwrap()
}

/////////////////////////////////////////////////////////////////////////


use wasm_bindgen_test::{wasm_bindgen_test_configure, wasm_bindgen_test};

wasm_bindgen_test_configure!(run_in_browser);


#[wasm_bindgen_test]
fn web_test() {
    println!("Hello!");
}
