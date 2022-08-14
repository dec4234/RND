use std::borrow::Borrow;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use crate::protocol::{from_bytes, IncomingHandler, OutgoingHandler, to_bytes};
use async_trait::async_trait;
use serde::Serialize;
use crate::{Encryptor, MessageSpec, Packet};
use anyhow::{anyhow, Result};
use rand_core::OsRng;
use ristretto255_dh::{EphemeralSecret, PublicKey, SharedSecret};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use crate::Packet::EnableEncryption;
use crate::packet::EnableEncryptionSpec;
use log::{info};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, Mutex};

pub struct Client {
    pub stream: TcpStream,
    pub client_secret: Option<EphemeralSecret>,
    pub client_public: Option<PublicKey>,
    pub client_shared: Option<SharedSecret>,
    pub encryptor: Option<Encryptor>,
}

impl Client {
    pub async fn new<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        Ok(Self {
            stream: TcpStream::connect(addr).await?,
            client_secret: None,
            client_public: None,
            client_shared: None,
            encryptor: None,
        })
    }

    pub async fn send_packet<S: Serialize>(&mut self, packet: S) -> Result<()> {
        if let Some(encryptor) = &self.encryptor {
            self.stream.write(encryptor.encrypt(serde_json::to_string(&packet)?).as_bytes()).await?;
        } else {
            self.stream.write(serde_json::to_string(&packet)?.as_bytes()).await?;
        }

        Ok(())
    }

    pub async fn enable_encryption(&mut self) -> Result<()> {
        self.client_secret = Some(EphemeralSecret::new(&mut OsRng));

        if let Some(secret) = self.client_secret.borrow() {
            self.client_public = Some(PublicKey::from(secret));
        }

        // 2. Send Tokens to Server
        self.send_packet(EnableEncryption(EnableEncryptionSpec {
            public: to_bytes(self.client_public.unwrap()),
        })).await?;

        // 3. Await response from the server
        let packet = self.read_next().await?;
        match packet {
            EnableEncryption(EnableEncryptionSpec { public }) => {
                if let Some(secret) = &self.client_secret {
                    self.client_shared = Some(secret.diffie_hellman(&from_bytes(public)));
                    self.encryptor = Some(Encryptor::from_bytes(public));
                }
            }
            _ => {
                return Err(anyhow!("Unexpected packet"));
            }
        }

        info!("Enabled encryption");

        Ok(())
    }
}

#[async_trait]
impl IncomingHandler for Client {
    async fn try_read(&mut self) -> Result<Packet> {
        let mut data = [0 as u8; 5096];

        if let Ok(size) = self.stream.try_read(&mut data) {
            let packet: Packet = serde_json::from_slice(&data[..size])?;

            return Ok(packet);
        }

        Err(anyhow!("No packet"))
    }

    async fn read_next(&mut self) -> Result<Packet> {
        let mut data = [0 as u8; 5096];

        if let Ok(size) = self.stream.read(&mut data).await {
            let packet: Packet = serde_json::from_slice(&data[..size])?;

            return Ok(packet);
        }

        Err(anyhow!("No packet"))
    }
}

#[async_trait]
impl OutgoingHandler for Client {
    async fn send_string(&mut self, s: String) -> Result<()> {
        self.send_packet(Packet::Message(MessageSpec {
            payload: s,
        })).await
    }
}


pub struct Server {
    listener: Arc<TcpListener>,
    pub connections: Arc<Mutex<Vec<Arc<Mutex<ClientConnection>>>>>,
}

impl Server {
    pub async fn new<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        Ok(Self {
            listener: Arc::new(TcpListener::bind(addr).await?),
            connections: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn accept_new_connections(&self) {
        let listener = self.listener.clone();
        let connections = self.connections.clone();

        tokio::spawn(async move {
            loop {
                let conn = listener.accept().await;

                if let Ok(conn) = conn {
                    let client = ClientConnection::new(conn.0); // Lost Info: Socket Address
                    ClientConnection::receive_packets(client.clone()).await;
                    connections.lock().await.push(client);
                }
            }
        });
    }
}

pub struct ClientConnection {
    pub conn: TcpStream,
    pub server_secret: Option<EphemeralSecret>,
    pub server_public: Option<PublicKey>,
    pub server_shared: Option<SharedSecret>,
    encryptor: Option<Encryptor>,
    pub channel: (Sender<Packet>, Receiver<Packet>),
}

impl ClientConnection {
    pub fn new(conn: TcpStream) -> Arc<Mutex<Self>> {
        Arc::new(
            Mutex::new(
                Self {
                    conn,
                    server_secret: None,
                    server_public: None,
                    server_shared: None,
                    encryptor: None,
                    channel: mpsc::channel(4096),
                }
            )
        )
    }

    pub async fn send_packet<S: Serialize>(&mut self, packet: S) -> Result<()> {
        if let Some(encryptor) = &self.encryptor {
            self.conn.write(encryptor.encrypt(serde_json::to_string(&packet)?).as_bytes()).await?;
        } else {
            self.conn.write(serde_json::to_string(&packet)?.as_bytes()).await?;
        }

        // self.conn.write(serde_json::to_string(&packet)?.as_bytes()).await?;

        Ok(())
    }

    pub async fn receive_packets(sel: Arc<Mutex<Self>>) {
        let send = sel.lock().await.channel.0.clone();

        tokio::spawn(async move {
            loop {
                let packet = sel.lock().await.try_read().await;
                if let Ok(packet) = packet {
                    send.send(packet).await.unwrap();
                }

                thread::sleep(Duration::from_millis(20)); // Sleep so mutex isn't locked all the time
            }
        });
    }

    async fn analyze_encryption_request(&mut self, packet: &Packet) {
        if let EnableEncryption(EnableEncryptionSpec { public }) = packet {
            // generate tokens and stuff here
            self.server_secret = Some(EphemeralSecret::new(&mut OsRng));

            if let Some(secret) = self.server_secret.borrow() {
                self.server_public = Some(PublicKey::from(secret));
            }

            if let Some(secret) = &self.server_secret {
                self.server_shared = Some(secret.diffie_hellman(&from_bytes(public.clone())));
                self.encryptor = Some(Encryptor::from_bytes(public.clone()));
            }

            // return server public key to the client
            self.send_packet(EnableEncryption(EnableEncryptionSpec {
                public: to_bytes(self.server_public.unwrap()),
            })).await.unwrap();
        }
    }
}

#[async_trait]
impl OutgoingHandler for ClientConnection {
    async fn send_string(&mut self, s: String) -> Result<()> {
        self.send_packet(Packet::Message(MessageSpec {
            payload: s,
        })).await
    }
}

#[async_trait]
impl IncomingHandler for ClientConnection {
    async fn try_read(&mut self) -> Result<Packet> {
        let mut data = [0 as u8; 5096];

        if let Ok(size) = self.conn.try_read(&mut data) {
            let packet: Packet = serde_json::from_slice(&data[..size])?;

            self.analyze_encryption_request(&packet).await;

            return Ok(packet);
        }

        Err(anyhow!("No packet"))
    }

    async fn read_next(&mut self) -> Result<Packet> {
        let mut data = [0 as u8; 5096];

        if let Ok(size) = self.conn.read(&mut data).await {
            let packet: Packet = serde_json::from_slice(&data[..size])?;

            self.analyze_encryption_request(&packet).await;

            return Ok(packet);
        }

        Err(anyhow!("No packet"))
    }
}

/// Used to identify the source of a packet
/// in a public collection channel
///
/// Unused
pub struct PacketContainer {

}