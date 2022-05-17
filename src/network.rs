use crate::protocol::{IncomingHandler, OutgoingHandler};
use async_trait::async_trait;
use serde::Serialize;
use crate::Packet;
use anyhow::Result;

pub struct Client {

}

impl Client {
    pub async fn send_packet<S: Serialize>(&mut self, packet: S) -> Result<()> {
        todo!()
    }
}

#[async_trait]
impl IncomingHandler for Client {
    fn try_next(&mut self) -> anyhow::Result<Packet> {
        todo!()
    }

    async fn read_next(&mut self) -> anyhow::Result<Packet> {
        todo!()
    }
}

#[async_trait]
impl OutgoingHandler for Client {
    async fn send_string(&mut self, s: String) -> anyhow::Result<()> {
        todo!()
    }
}


pub struct Server {

}

impl Server {
    pub async fn send_packet<S: Serialize>(&mut self, packet: S) -> Result<()> {
        todo!()
    }
}