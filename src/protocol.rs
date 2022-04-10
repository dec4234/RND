use std::io::Write;
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::mpsc;
use std::thread;
use anyhow::Result;
use serde::Serialize;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        Ok(Self {
            stream: TcpStream::connect(addr)?,
        })
    }

    pub fn send_packet<S: Serialize>(&mut self, packet: S) -> Result<()> {

        self.stream.write(serde_json::to_string(&packet)?.as_bytes())?;

        Ok(())
    }
}

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new<A: ToSocketAddrs>(addrs: A) -> Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(addrs)?,
        })
    }

    pub fn start_listening(&mut self) -> Result<()> {

        let mpsc = mpsc::channel();

        thread::spawn(move || {

        });

        Ok(())
    }
}