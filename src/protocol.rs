use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        Ok(Self {
            stream: TcpStream::connect(addr)?,
        })
    }

    pub fn send_string(&mut self, s: String) -> Result<()> {
        self.stream.write(s.as_bytes())?;

        Ok(())
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

    // pub fn start_listening<'a, P: Deserialize<'a>>(mut self, sender: Sender<P>) -> Result<()> {

    pub fn start_listening(self, sender: Sender<String>) -> Result<()> {
        thread::spawn(move || {
            loop {
                for s in self.listener.incoming() {
                    if let Ok(mut s) = s {
                        let mut data = [0 as u8; 5096];

                        while match s.read(&mut data) {
                            Ok(size) => {

                                sender.clone().send(String::from_utf8_lossy(&data[0..size]).to_string()).unwrap();

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
            }
        });

        Ok(())
    }
}