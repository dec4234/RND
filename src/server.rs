use std::sync::Arc;
use std::task::Context;
use serde::Deserialize;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use anyhow::{anyhow, Result};
use tokio::io::AsyncReadExt;
use crate::Packet1::Packet1;

pub struct Server {
    listener: TcpListener,
    pub connections: Arc<Mutex<Vec<ClientConnection>>>,
}

impl Server {
    pub async fn new<A: ToSocketAddrs>(addrs: A) -> Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(addrs).await?,
            connections: Arc::new(Mutex::new(Vec::new())),
        })
    }

    // pub fn start_listening<'a, P: Deserialize<'a>>(mut self, sender: Sender<P>) -> Result<()> {

    pub async fn accept(&mut self) -> Result<ClientConnection> {
        let con = self.listener.accept().await;

        /*
        if let Ok(con) = con {
            self.connections.lock().await.push(ClientConnection::new(con.0));
        }
         */

        Ok(ClientConnection::new(con?.0))
    }

    pub async fn accept_incoming<'a, P: Deserialize<'a>>(&mut self, sender: Sender<P>) -> Result<()> {
        loop {
            self.connections.lock().await.push(ClientConnection::new(self.listener.accept().await.unwrap().0));
        }

        Ok(())
    }

    pub fn relay_packets<'a, P: Deserialize<'a>>(self, sender: Sender<P>) -> Result<()> {
        /*
                    let mut data = [0 as u8; 5096];

                    while match s.read(&mut data.clone()) {
                        Ok(size) => {
                            sender.clone().send(serde_json::from_str::<P>(String::from_utf8_lossy(&data.clone()[0..size]).clone()).unwrap()).unwrap();

                            true
                        }
                        Err(_) => {
                            println!("Terminating: {}", s.peer_addr().unwrap());
                            s.shutdown(Shutdown::Both).unwrap();
                            false
                        }
                    } {}

                    data = [0 as u8; 5096]; // Reset Buffer
         */

        Ok(())
    }
}

pub struct ClientConnection {
    pub conn: TcpStream,
}

impl ClientConnection {
    pub fn new(conn: TcpStream) -> Self {
        Self {
            conn,
        }
    }

    pub async fn read_incoming_packet(&mut self) -> Result<Packet1> {
        let mut data = [0 as u8; 5096];

        let r = self.conn.read(&mut data).await;

        if let Ok(size) = r {
            return Ok(deserialize_raw::<Packet1>(&data, size)?);
        }

        Err(anyhow!("No packet"))
    }
}

pub fn deserialize_raw<'a, P: Deserialize<'a>>(buf: &'a [u8], size: usize) -> serde_json::Result<P> {
    // let s: &str = String::from_utf8_lossy(&buf[0..size]).clone().as_ref();

    serde_json::Result::Ok(serde_json::from_slice(&buf[0..size])).unwrap()
}