use tokio::net::{TcpStream, ToSocketAddrs};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub async fn new<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        Ok(Self {
            stream: TcpStream::connect(addr).await?,
        })
    }

    pub async fn send_string(&mut self, s: String) -> Result<()> {
        self.stream.write(s.as_bytes()).await?;

        Ok(())
    }

    pub async fn send_packet<S: Serialize>(&mut self, packet: S) -> Result<()> {
        self.stream.write(serde_json::to_string(&packet)?.as_bytes()).await?;

        Ok(())
    }
}