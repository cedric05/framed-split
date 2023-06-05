use std::{error::Error, time::Duration};

use futures::prelude::*;
use test_simple::Message;
use tokio::net::{TcpListener, TcpStream};
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
        println!("[server] [info] [writer] writing message {:?}", message);
        writer.send(message.clone()).await.unwrap();
        message.server_increment();
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
            Some(Ok(some)) => println!("[server] [info] [reader] {:?}", some),
            Some(Err(error)) => println!("[server] [error] [reader] ran into error {}", error),
            None => println!("[server] [warn] [reader] no message recieved"),
        }
    }
}

async fn run_main(stream: TcpStream) {
    let (reader, writer) = stream.into_split();
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

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let socket = TcpListener::bind("127.0.0.1:8000").await.unwrap();

    loop {
        let (stream, _addr) = socket.accept().await?;
        tokio::spawn(run_main(stream));
    }
}
