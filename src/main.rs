use std::borrow::BorrowMut;
use std::io::{Error, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use rand_core::OsRng;
use crate::encrypt::{Encryptor};
use uuid::Uuid;
use ristretto255_dh::{EphemeralSecret, PublicKey};
use crate::protocol::{PacketDirection};
use serde::{Serialize, Deserialize};
use crate::client::Client;
use crate::packet::MessageSpec;
use crate::packet::Packet;
use crate::server::Server;

pub mod encrypt;
pub mod protocol;
pub mod packet;
pub mod server;
pub mod client;
pub mod network;

#[tokio::main]
async fn main() {
    env_logger::init();

    start_loop().await;
}

/*
Need to make read_next not async to support a 1 call enable_encryption request
 */
async fn start_loop() {
    let mut s = Server::new("127.0.0.1:27893").await.unwrap();
    let mut c = Client::new("127.0.0.1:27893").await.unwrap();

    let mut cc = s.accept().await.unwrap();

    // Enable encryption
    c.enable_encryption().await.unwrap();
    cc.enable_encryption().await.unwrap();
    c.enable_encryption_final().await.unwrap();

    loop {
        c.send_packet(Packet::Message(MessageSpec {
            payload: String::from("Hey lhbglhasbgfhbldjfbgljrhbgtilhbgljvbljbfg")
        })).await.unwrap();

        let p = cc.read_incoming_packet().await.unwrap();

        match p {
            Packet::Handshake(payload) => {
                println!("{}", payload.payload);
            }
            Packet::Message(payload) => {
                println!("{}", payload.payload);
            }
            _ => {}
        }

        thread::sleep(Duration::from_secs(3));
    }
}


