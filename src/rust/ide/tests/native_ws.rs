use ide_view::prelude::*;
use json_rpc::{Transport, TransportEvent};
use futures::channel::mpsc::UnboundedSender;
use tungstenite::Error;

use futures::{future, pin_mut, StreamExt, Stream, Sink, SinkExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use tokio::net::TcpStream;
use utils::fail::FallibleResult;
use ast::prelude::fmt::Formatter;
use std::pin::Pin;
use std::sync::Arc;

const ADDRESS : &str = "ws://localhost:30615";

type WSStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct Socket {
    tx: Pin<Box<dyn Sink<Message,Error=Error>>>,
    rx: Box<dyn Stream<Item=Result<Message,Error>>>,
}

impl Socket {
    fn new(ws:WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        let (tx,rx) = ws.split();
        Self {
            tx : Box::pin(tx),
            rx : Box::new(rx),
        }
    }

    pub async fn new_opened
    (parent:impl AnyLogger, url:&str) -> FallibleResult<Self> {
        let (stream,_) = connect_async(url).await?;
        Ok(Self::new(stream))
    }
}

struct WS {
    socket : Arc<RefCell<Socket>>
}

impl WS {
    pub async fn new_opened(parent:impl AnyLogger, url:&str) -> FallibleResult<Self> {
        let socket = Socket::new_opened(parent,url).await?;
        Ok(Self {
            socket : Arc::new(RefCell::new(socket))
        })
    }

    fn send_blocking(&self, message:Message) -> FallibleResult<()> {
        let weak = Arc::downgrade(&self.socket);
        tokio::runtime::Handle::current().spawn(async move {
            // if let Some(this) = weak.upgrade() {
            //     this.borrow_mut().tx.send(message).await.map_err(Into::into)
            // } else {
            //     Err(failure::format_err!("WebSocket already dropped!"))
            // }
        });
        Ok(())
        //tokio::runtime::Handle::current().block_on(send_future).map_err(Into::into)
    }
}

impl Debug for WS {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl Transport for WS {
    fn send_text(&mut self, message: &str) -> Result<(), failure::Error> {
        let message = Message::text(message);
        self.send_blocking(message)
    }

    fn send_binary(&mut self, message: &[u8]) -> Result<(), failure::Error> {
        let message = Message::binary(message);
        self.send_blocking(message)
    }

    fn set_event_transmitter(&mut self, transmitter: UnboundedSender<TransportEvent>) {
        todo!()
    }
}


#[tokio::test]
async fn moje() {
    println!("Will connect to {}",ADDRESS);
    let mut ws = WS::new_opened(DefaultTraceLogger::new("WS"), ADDRESS).await.unwrap();
    println!("Connection established!");
    ws.send_text("Hello");
}


async fn inner() {
    println!("Will connect to {}",ADDRESS);
    let url = url::Url::parse(&ADDRESS).unwrap();

    let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_tx));

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();
            tokio::io::stdout().write_all(&data).await.unwrap();
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

#[test]
fn main2() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            inner().await
        })
}

#[tokio::test]
async fn main() {
    inner().await
    // println!("Will connect to {}",ADDRESS);
    // let url = url::Url::parse(&ADDRESS).unwrap();
    //
    // let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded();
    // tokio::spawn(read_stdin(stdin_tx));
    //
    // let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    // println!("WebSocket handshake has been successfully completed");
    //
    // let (write, read) = ws_stream.split();
    //
    // let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    // let ws_to_stdout = {
    //     read.for_each(|message| async {
    //         //println!("New message: {:?}",message);
    //         let data = message.unwrap().into_data();
    //         println!("New message: {:?}",data);
    //         // tokio::io::stdout().write_all(&data).await.unwrap();
    //         future::ready(()).await
    //     })
    // };
    //
    // pin_mut!(stdin_to_ws, ws_to_stdout);
    // future::select(stdin_to_ws, ws_to_stdout).await;
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
async fn read_stdin(tx: futures::channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        let text = String::from_utf8_lossy(&buf);
        println!("Will send {}", text);
        tx.unbounded_send(Message::text(text)).unwrap();
    }
}
