use std::borrow::Borrow;
use tokio::net::{TcpStream, ToSocketAddrs};
use anyhow::{anyhow, Result};
use rand_core::OsRng;
use ristretto255_dh::{EphemeralSecret, PublicKey, SharedSecret};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::{Encryptor, packet};
use crate::packet::{EnableEncryptionSpec, Packet as Packet1};
use crate::packet::Packet::EnableEncryption;
use crate::protocol::{from_bytes, to_bytes};
use crate::server::deserialize_raw;

pub struct Client {
    conn: ServerConnection,
}

impl Client {
    pub async fn new<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        Ok(Self {
            conn: ServerConnection::new(TcpStream::connect(addr).await?)
        })
    }

    pub async fn send_string(&mut self, s: String) -> Result<()> {
        self.conn.stream.write(s.as_bytes()).await?;

        Ok(())
    }

    pub async fn send_packet<S: Serialize>(&mut self, packet: S) -> Result<()> {
        self.conn.stream.write(serde_json::to_string(&packet)?.as_bytes()).await?;

        Ok(())
    }

    pub async fn read_next(&mut self) -> Result<Packet1> {
        let mut data = [0 as u8; 5096];

        let r = self.conn.stream.read(&mut data).await;

        if let Ok(size) = r {
            return Ok(deserialize_raw::<Packet1>(&data, size)?);
        }

        Err(anyhow!("No packet"))
    }

    pub async fn enable_encryption(&mut self) -> Result<()> {

        // 1. Generate Tokens
        self.conn.client_secret = Some(EphemeralSecret::new(&mut OsRng));

        if let Some(secret) = self.conn.client_secret.borrow() {
            self.conn.client_public = Some(PublicKey::from(secret));
        }

        // 2. Send Tokens to Server
        self.send_packet(EnableEncryption(EnableEncryptionSpec {
            public: to_bytes(self.conn.client_public.unwrap()),
        })).await?;

        Ok(())
    }

    pub async fn enable_encryption_final(&mut self) -> Result<()> {
        // 3. Receive Encryption Packet from server and set shared secret
        let pack = self.read_next().await?;

        if let EnableEncryption(p) = pack {
            let server_public = from_bytes(p.public);

            if let Some(secret) = &self.conn.client_secret {
                self.conn.client_shared = Some(secret.diffie_hellman(&server_public));
                self.conn.encryptor = Some(Encryptor::from_bytes(p.public));
            }
        } else {
            return Err(anyhow!("Next Packet was not an Encryption Request!"));
        }

        Ok(())
    }

    pub async fn test_encryption(&mut self) -> Result<()> {
        todo!()
    }
}

pub struct ServerConnection {
    pub stream: TcpStream,
    pub client_secret: Option<EphemeralSecret>,
    pub client_public: Option<PublicKey>,
    pub client_shared: Option<SharedSecret>,
    pub encryptor: Option<Encryptor>,
}

impl ServerConnection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            client_secret: None,
            client_public: None,
            client_shared: None,
            encryptor: None,
        }
    }
}