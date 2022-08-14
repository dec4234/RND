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
use crate::protocol::{OutgoingHandler, PacketDirection};
use serde::{Serialize, Deserialize};
use crate::network::{Client, Server};
use crate::packet::MessageSpec;
use crate::packet::Packet;

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
    s.accept_new_connections();
    let mut c = Client::new("127.0.0.1:27893").await.unwrap();
    c.enable_encryption().await.unwrap();

    let sc = s.connections.clone();

    loop {
        c.send_string(String::from("ABCDEFGHIJKLMNOPQRSTUVWXYZ123456789")).await.unwrap();

        let lock = sc.lock().await;

        if let Some(conn) = lock.get(0) {
            let r = &mut conn.lock().await.channel.1;

            if let Ok(packet) = r.try_recv() {
                match packet {
                    Packet::Handshake(_) => {}
                    Packet::Message(m) => {
                        println!("{}", m.payload);
                    }
                    Packet::EnableEncryption(_) => {}
                }
            }
        }

        thread::sleep(Duration::from_millis(1000));
    }
}


