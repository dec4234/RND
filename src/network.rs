use std::borrow::Borrow;
use crate::protocol::{from_bytes, IncomingHandler, OutgoingHandler, to_bytes};
use async_trait::async_trait;
use serde::Serialize;
use crate::{Encryptor, Packet};
use anyhow::{anyhow, Result};
use rand_core::OsRng;
use ristretto255_dh::{EphemeralSecret, PublicKey, SharedSecret};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use crate::Packet::EnableEncryption;
use crate::packet::EnableEncryptionSpec;

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
        self.stream.write(serde_json::to_string(&packet)?.as_bytes()).await?;

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
                    self.encryptor = Some(Encryptor::from_bytes(public)); // Figure out how to plug SharedSecret into encryptor
                }
            }
            _ => {
                return Err(anyhow!("Unexpected packet"));
            }
        }

        Ok(())
    }
}

#[async_trait]
impl IncomingHandler for Client {
    fn try_read(&mut self) -> Result<Packet> {
        let mut data = [0 as u8; 5096];

        if let Ok(size) = self.stream.try_read(&mut data) {
            let packet: Packet = serde_json::from_slice(&data[..size])?;

            return Ok(packet);
        }

        Err(anyhow!("No packet"))
    }

    async fn read_next(&mut self) -> Result<Packet> {
        todo!()
    }
}

#[async_trait]
impl OutgoingHandler for Client {
    async fn send_string(&mut self, s: String) -> Result<()> {
        todo!()
    }
}


pub struct Server {
    listener: TcpListener,
    connections: Vec<ClientConnection>,
}

impl Server {
    pub async fn new<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(addr).await?,
            connections: Vec::new(),
        })
    }
}

pub struct ClientConnection {
    pub conn: TcpStream,
    pub server_secret: Option<EphemeralSecret>,
    pub server_public: Option<PublicKey>,
    pub server_shared: Option<SharedSecret>,
    encryptor: Option<Encryptor>,
}

impl ClientConnection {
    pub async fn send_packet<S: Serialize>(&mut self, packet: S) -> Result<()> {
        if let Some(encryptor) = &self.encryptor {
            self.conn.write(encryptor.encrypt(serde_json::to_string(&packet)?).as_bytes());
        } else {
            self.conn.write(serde_json::to_string(&packet)?.as_bytes()).await?;
        }

        self.conn.write(serde_json::to_string(&packet)?.as_bytes()).await?;

        Ok(())
    }

    async fn analyze_encryption_request(&mut self, packet: Packet) {
        if let EnableEncryption(EnableEncryptionSpec { public }) = packet {
            // generate tokens and stuff here

            self.server_secret = Some(EphemeralSecret::from(public));
            // self.server_public = Some(PublicKey::from(self.server_secret.unwrap()));
            // self.server_shared = Some(SharedSecret::from(self.server_secret.unwrap(), self.server_public.unwrap()));
            // self.encryptor = Some(Encryptor::new(self.server_shared.unwrap()));

            // return server public key to the client
        }
    }
}

#[async_trait]
impl OutgoingHandler for ClientConnection {
    async fn send_string(&mut self, s: String) -> Result<()> {
        todo!()
    }
}

#[async_trait]
impl IncomingHandler for ClientConnection {
    fn try_read(&mut self) -> Result<Packet> {
        todo!()
    }

    async fn read_next(&mut self) -> Result<Packet> {
        todo!()
    }
}