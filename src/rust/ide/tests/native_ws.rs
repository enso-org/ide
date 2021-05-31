// #![feature(async_closure)]
// #![feature(unboxed_closures)]
// #![feature(fn_traits)]
//
// use ide_view::prelude::*;
// use json_rpc::{Transport, TransportEvent};
// use futures::channel::mpsc::{UnboundedSender, UnboundedReceiver};
// use tungstenite::Error;
//
// use futures::{future, pin_mut, StreamExt, Stream, Sink, SinkExt};
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
// use tokio::net::TcpStream;
// use utils::fail::FallibleResult;
// use ast::prelude::fmt::Formatter;
// use std::pin::Pin;
// use std::sync::{Arc, Mutex};
// use futures::FutureExt;
// use std::future::Future;
// use futures::stream::SplitSink;
// use ide::prelude::BoxFuture;
//
// const ADDRESS : &str = "ws://localhost:30615";
//
// type WSStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
//
// struct Socket {
//     // ws_tx: Pin<Box<dyn Sink<Message,Error=Error> + Send>>,
//     // ws_rx: Box<dyn Stream<Item=Result<Message,Error>> + Send>,
//
//     // input: UnboundedReceiver<Message>, // Here go messages, which are then pushed to the socket.
//     // out: UnboundedSender<TransportEvent>,
// }
//
// // impl Socket {
// //     fn new(ws:WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
// //
// //     }
// //
// //     pub async fn new_opened
// //     (parent:impl AnyLogger, url:&str) -> FallibleResult<Self> {
// //         let (stream,_) = connect_async(url).await?;
// //         Ok(Self::new(stream))
// //     }
// //}
//
// struct WS {
//     to_socket : UnboundedSender<Message>,
//     to_client : Arc<Mutex<UnboundedSender<TransportEvent>>>,
// }
//
// fn process<Fut:Future<Output=()>>(mut f:impl FnMut(Message) -> Fut) {
//     f(todo!());
// }
//
// struct Foo {
//     ws_tx:SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
// }
//
// impl FnOnce<(Message,)> for Foo {
//     type Output = BoxFuture<'static,()>;
//
//     extern "rust-call" fn call_once(mut self, args: (Message,)) -> Self::Output {
//         self.call_mut(args)
//     }
// }
//
// impl FnMut<(Message,)> for Foo {
//     extern "rust-call" fn call_mut(&mut self, msg:(Message,)) -> Self::Output {
//         //self.ws_tx.clone().send(msg).await
//         self.ws_tx.send(msg.0).map(|r| r.unwrap()).boxed()
//     }
// }
//
// impl WS {
//     pub async fn new_opened(parent:impl AnyLogger, url:&str) -> FallibleResult<Self> {
//         //let socket = Socket::new_opened(parent,url).await?;
//         let (stream,_) = connect_async(url).await?;
//         let (mut ws_tx, mut ws_rx) = stream.split();
//
//
//
//
//         let rt = tokio::runtime::Handle::current();
//
//         // Sending messages
//         let to_socket = {
//             let mut ws_tx = Arc::new(Mutex::new(ws_tx));
//              let (tx,rx) = futures::channel::mpsc::unbounded();
//             // process(Foo{ws_tx});
//             let processor = async move |msg| {
//                 ws_tx.lock().unwrap().send(msg).await;
//             };
//
//             process(processor);
//             // rt.spawn(rx.for_each(move |msg| {t `FnMut` derives from here
//             //     ws_tx.send(msg).map(Result::unwrap)
//             // }));
//             tx
//         };
//
//         // Handling messages
//         let (to_client,_) = futures::channel::mpsc::unbounded();
//         let to_client = Arc::new(Mutex::new(to_client));
//         {
//             let to_client = Arc::downgrade(&to_client);
//
//             // Handle incoming
//             // let processor = ws_rx.for_each(async move |msg| {
//             //     let event = match msg {
//             //         Ok(Message::Text(s)) => TransportEvent::TextMessage(s),
//             //         Ok(Message::Binary(b)) => TransportEvent::BinaryMessage(b),
//             //         Ok(Message::Close(_)) => TransportEvent::Closed,
//             //         Ok(_) => return (),
//             //         Err(e) => TransportEvent::Closed,
//             //     };
//             //     if let Some(sink) = to_client.upgrade() {
//             //         if let Ok(sink) = sink.lock() {
//             //             sink.unbounded_send(event).unwrap();
//             //         }
//             //     }
//             // });
//             // rt.spawn(processor);
//         }
//         Ok(Self {to_socket,to_client})
//     }
//
//     fn send_blocking(&self, message:Message) -> FallibleResult<()> {
//         let weak = Arc::downgrade(&self.socket);
//         tokio::runtime::Handle::current().spawn(async move {
//             if let Some(this) = weak.upgrade() {
//                 this.lock().unwrap().ws_tx.send(message).await.map_err(Into::into)
//             } else {
//                 Err(failure::format_err!("WebSocket already dropped!"))
//             }
//         });
//         Ok(())
//         //tokio::runtime::Handle::current().block_on(send_future).map_err(Into::into)
//     }
// }
//
// impl Debug for WS {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         todo!()
//     }
// }
//
// impl Transport for WS {
//     fn send_text(&mut self, message: &str) -> Result<(), failure::Error> {
//         let message = Message::text(message);
//         self.send_blocking(message)
//     }
//
//     fn send_binary(&mut self, message: &[u8]) -> Result<(), failure::Error> {
//         let message = Message::binary(message);
//         self.send_blocking(message)
//     }
//
//     fn set_event_transmitter(&mut self, transmitter: UnboundedSender<TransportEvent>) {
//         todo!()
//     }
// }
//
//
// #[tokio::test]
// async fn moje() {
//     println!("Will connect to {}",ADDRESS);
//     let mut ws = WS::new_opened(DefaultTraceLogger::new("WS"), ADDRESS).await.unwrap();
//     println!("Connection established!");
//     ws.send_text("Hello");
// }
//
//
// async fn inner() {
//     println!("Will connect to {}",ADDRESS);
//     let url = url::Url::parse(&ADDRESS).unwrap();
//
//     let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded();
//     tokio::spawn(read_stdin(stdin_tx));
//
//     let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
//     println!("WebSocket handshake has been successfully completed");
//
//     let (write, read) = ws_stream.split();
//
//     let stdin_to_ws = stdin_rx.map(Ok).forward(write);
//     let ws_to_stdout = {
//         read.for_each(|message| async {
//             let data = message.unwrap().into_data();
//             tokio::io::stdout().write_all(&data).await.unwrap();
//         })
//     };
//
//     pin_mut!(stdin_to_ws, ws_to_stdout);
//     future::select(stdin_to_ws, ws_to_stdout).await;
// }
//
// #[test]
// fn main2() {
//     tokio::runtime::Builder::new_multi_thread()
//         .enable_all()
//         .build()
//         .unwrap()
//         .block_on(async {
//             inner().await
//         })
// }
//
// #[tokio::test]
// async fn main() {
//     inner().await
//     // println!("Will connect to {}",ADDRESS);
//     // let url = url::Url::parse(&ADDRESS).unwrap();
//     //
//     // let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded();
//     // tokio::spawn(read_stdin(stdin_tx));
//     //
//     // let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
//     // println!("WebSocket handshake has been successfully completed");
//     //
//     // let (write, read) = ws_stream.split();
//     //
//     // let stdin_to_ws = stdin_rx.map(Ok).forward(write);
//     // let ws_to_stdout = {
//     //     read.for_each(|message| async {
//     //         //println!("New message: {:?}",message);
//     //         let data = message.unwrap().into_data();
//     //         println!("New message: {:?}",data);
//     //         // tokio::io::stdout().write_all(&data).await.unwrap();
//     //         future::ready(()).await
//     //     })
//     // };
//     //
//     // pin_mut!(stdin_to_ws, ws_to_stdout);
//     // future::select(stdin_to_ws, ws_to_stdout).await;
// }
//
// // Our helper method which will read data from stdin and send it along the
// // sender provided.
// async fn read_stdin(tx: futures::channel::mpsc::UnboundedSender<Message>) {
//     let mut stdin = tokio::io::stdin();
//     loop {
//         let mut buf = vec![0; 1024];
//         let n = match stdin.read(&mut buf).await {
//             Err(_) | Ok(0) => break,
//             Ok(n) => n,
//         };
//         buf.truncate(n);
//         let text = String::from_utf8_lossy(&buf);
//         println!("Will send {}", text);
//         tx.unbounded_send(Message::text(text)).unwrap();
//     }
// }
