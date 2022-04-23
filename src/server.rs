use std::borrow::Borrow;
use std::sync::Arc;
use std::task::Context;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use anyhow::{anyhow, Result};
use rand_core::OsRng;
use ristretto255_dh::{EphemeralSecret, PublicKey, SharedSecret};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::Packet;
use crate::Packet1::{EnableEncryptionSpec, Packet1};
use crate::Packet::EnableEncryption;
use crate::protocol::{from_bytes, to_bytes};

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
}

pub struct ClientConnection {
    pub conn: TcpStream,
    pub server_secret: Option<EphemeralSecret>,
    pub server_public: Option<PublicKey>,
    pub server_shared: Option<SharedSecret>,
}

impl ClientConnection {
    pub fn new(conn: TcpStream) -> Self {
        Self {
            conn,
            server_secret: None,
            server_public: None,
            server_shared: None,
        }
    }

    pub async fn send_string(&mut self, s: String) -> Result<()> {
        self.conn.write(s.as_bytes()).await?;

        Ok(())
    }

    pub async fn send_packet<S: Serialize>(&mut self, packet: S) -> Result<()> {
        self.conn.write(serde_json::to_string(&packet)?.as_bytes()).await?;

        Ok(())
    }

    pub async fn read_incoming_packet(&mut self) -> Result<Packet1> {
        let mut data = [0 as u8; 5096];

        let r = self.conn.read(&mut data).await;

        if let Ok(size) = r {
            return Ok(deserialize_raw::<Packet1>(&data, size)?);
        }

        Err(anyhow!("No packet"))
    }

    pub async fn enable_encryption(&mut self) -> Result<()> {

        self.server_secret = Some(EphemeralSecret::new(&mut OsRng));

        if let Some(secret) = self.server_secret.borrow() {
            self.server_public = Some(PublicKey::from(secret));
        }

        let pack = self.read_incoming_packet().await?;

        if let EnableEncryption(p) = pack {
            let client_public = from_bytes(p.public);

            if let Some(secret) = &self.server_secret {
                self.server_shared = Some(secret.diffie_hellman(&client_public));
            }
            
            self.send_packet(Packet::EnableEncryption(EnableEncryptionSpec {
                public: to_bytes(self.server_public.unwrap())
            })).await?;
        } else {
            return Err(anyhow!("Next Packet was not an Encryption Request!"));
        }

        Ok(())
    }
}

pub fn deserialize_raw<'a, P: Deserialize<'a>>(buf: &'a [u8], size: usize) -> serde_json::Result<P> {
    // let s: &str = String::from_utf8_lossy(&buf[0..size]).clone().as_ref();

    serde_json::Result::Ok(serde_json::from_slice(&buf[0..size])).unwrap()
}