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
    /*
    let server_secret = EphemeralSecret::new(&mut OsRng);
    let server_public = PublicKey::from(&server_secret);

    let client_secret = EphemeralSecret::new(&mut OsRng);
    let client_public = PublicKey::from(&client_secret);

    let server_shared = server_secret.diffie_hellman(&client_public);
    let client_shared = client_secret.diffie_hellman(&server_public);

    let u: [u8; 32] = client_shared.try_into().unwrap();

    let s = String::from_utf8_lossy(&u[..]).to_string();

    thread::spawn(|| {
        let listener = TcpListener::bind("127.0.0.1:27893").unwrap();

        let u: [u8; 32] = server_shared.try_into().unwrap();

        let s = String::from_utf8_lossy(&u[..]).to_string();

        let d = Encryptor::new(s);

        for s in listener.incoming() {
            if let Ok(mut s) = s {
                let mut data = [0 as u8; 5096];

                while match s.read(&mut data) {
                    Ok(size) => {

                        println!("{}", d.decrypt(String::from_utf8_lossy(&data[0..size]).to_string()).unwrap().to_string());
                        // println!("{}", serde_json::from_str::<Packet>(&String::from_utf8_lossy(&data[0..size])).unwrap().title);

                        data = [0 as u8; 5096];

                        true
                    },
                    Err(_) => {
                        println!("Terminating: {}", s.peer_addr().unwrap());
                        s.shutdown(Shutdown::Both).unwrap();
                        false
                    }
                } {}
            }
        }
    });

    thread::spawn(move || {
        let msg = "This message is used as encryption testing. .-.-.----!".to_string();

        let mut connect = TcpStream::connect("127.0.0.1:27893").unwrap();

        loop {
            let e = Encryptor::new(s.clone().to_string());

            connect.write(e.encrypt(msg.clone()).as_bytes()).unwrap();


            /*
            let p = Packet::new("Test Packet".to_string(), "this is a test packet using serialization, deserialization and clone".to_string());

            connect.write(serde_json::to_string(&p).unwrap().as_bytes()).unwrap();
             */

            thread::sleep(Duration::from_secs(5));
        }
    });
     */

    /*
    let mpsc = tokio::sync::mpsc::channel::<Packet>(256);

    s.accept_incoming(mpsc.0).await.unwrap();

    for m in mpsc.1.iter() {
        println!("{}", m);
    }

    loop {
        c.send_string("ablgiblfkgbkgbrkjbgkrbkrbglkrjbkgbrkjbrklrbkgbrjgfkbgf".to_string()).await.unwrap();
        thread::sleep(Duration::from_secs(5));
    }
    */

    start_loop().await;

    loop {
        let p = Packet::Message(MessageSpec {
            payload: String::from("")
        });


    }
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


