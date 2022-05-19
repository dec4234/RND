use crate::protocol::{IncomingHandler, OutgoingHandler};
use async_trait::async_trait;
use serde::Serialize;
use crate::{Encryptor, Packet};
use anyhow::Result;
use ristretto255_dh::{EphemeralSecret, PublicKey, SharedSecret};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

pub struct Client {
    pub stream: TcpStream,
    pub client_secret: Option<EphemeralSecret>,
    pub client_public: Option<PublicKey>,
    pub client_shared: Option<SharedSecret>,
    pub encryptor: Option<Encryptor>,
}

impl Client {
    pub async fn send_packet<S: Serialize>(&mut self, packet: S) -> Result<()> {
        self.stream.write(serde_json::to_string(&packet)?.as_bytes()).await?;

        Ok(())
    }
}

#[async_trait]
impl IncomingHandler for Client {
    fn try_read(&mut self) -> Result<Packet> {
        todo!()
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
}

impl Server {

}

#[async_trait]
impl IncomingHandler for Server {
    fn try_read(&mut self) -> Result<Packet> {
        todo!()
    }

    async fn read_next(&mut self) -> Result<Packet> {
        todo!()
    }
}

pub struct Connection {
    pub conn: TcpStream,
    pub server_secret: Option<EphemeralSecret>,
    pub server_public: Option<PublicKey>,
    pub server_shared: Option<SharedSecret>,
}

impl Connection {
    pub async fn send_packet<S: Serialize>(&mut self, packet: S) -> Result<()> {
        self.conn.write(serde_json::to_string(&packet)?.as_bytes()).await?;

        Ok(())
    }
}

#[async_trait]
impl OutgoingHandler for Connection {
    async fn send_string(&mut self, s: String) -> Result<()> {
        todo!()
    }
}