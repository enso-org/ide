use ide_view::prelude::*;
use json_rpc::{Transport, TransportEvent};
use futures::channel::mpsc::UnboundedSender;
use failure::Error;

use futures::{future, pin_mut, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

const ADDRESS : &str = "ws://localhost:30615";

#[derive(Debug)]
struct WS {

}

impl Transport for WS {
    fn send_text(&mut self, message: &str) -> Result<(), Error> {
        todo!()
    }

    fn send_binary(&mut self, message: &[u8]) -> Result<(), Error> {
        todo!()
    }

    fn set_event_transmitter(&mut self, transmitter: UnboundedSender<TransportEvent>) {
        todo!()
    }
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
