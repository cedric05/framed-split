use std::time::Duration;

use futures::prelude::*;
use test_simple::Message;
use tokio::net::TcpStream;
use tokio_serde::formats::*;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub async fn writer_loop(
    mut writer: tokio_serde::Framed<
        FramedWrite<tokio::net::tcp::OwnedWriteHalf, LengthDelimitedCodec>,
        Message,
        Message,
        Json<Message, Message>,
    >,
) {
    let mut message = Message {
        msg: "some message".to_string(),
        server_count: 0,
        client_count: 0,
    };
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("[client] [info] [writer] writing message {:?}", message);
        writer.send(message.clone()).await.unwrap();
        message.client_increment();
    }
}

pub async fn reader_loop(
    mut reader: tokio_serde::Framed<
        FramedRead<tokio::net::tcp::OwnedReadHalf, LengthDelimitedCodec>,
        Message,
        Message,
        Json<Message, Message>,
    >,
) {
    loop {
        match reader.next().await {
            Some(Ok(some)) => println!("[client] [info] [reader] {:?}", some),
            Some(Err(error)) => println!("[client] [error] [reader] ran into error {}", error),
            None => println!("[client] [warn] [reader] no message recieved"),
        }
    }
}

#[tokio::main]
pub async fn main() {
    let socket = TcpStream::connect("127.0.0.1:8000").await.unwrap();
    let (reader, writer) = socket.into_split();
    let length_delimited_reader = FramedRead::new(reader, LengthDelimitedCodec::new());
    let length_delimited_writer = FramedWrite::new(writer, LengthDelimitedCodec::new());

    tokio::join!(
        reader_loop(tokio_serde::SymmetricallyFramed::new(
            length_delimited_reader,
            SymmetricalJson::default()
        )),
        writer_loop(tokio_serde::SymmetricallyFramed::new(
            length_delimited_writer,
            SymmetricalJson::default()
        ))
    );
}
