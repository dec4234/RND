use std::io::{Error, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use crate::encrypt::Encryptor;
use uuid::Uuid;

pub mod encrypt;

#[tokio::main]
async fn main() {
    thread::spawn(|| {
        let listener = TcpListener::bind("127.0.0.1:27893").unwrap();

        for s in listener.incoming() {
            if let Ok(mut s) = s {
                let mut data = [0 as u8; 50];

                while match s.read(&mut data) {
                    Ok(size) => {

                        println!("{}", String::from_utf8_lossy(&data));

                        data = [0 as u8; 50];

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
    });

    thread::spawn(|| {
        let msg = "Hello this is a message that is very long hello hello!!!".to_string();

        let mut connect = TcpStream::connect("127.0.0.1:27893").unwrap();

        loop {
            let e = Encryptor::new(Uuid::new_v4().to_string());

            connect.write(e.encrypt(msg.clone()).as_bytes()).unwrap();

            thread::sleep(Duration::from_secs(5));
        }
    });

    loop {

    }
}
