use std::{future::Future, net::TcpStream, io::{Read, BufReader, BufRead}, task::Poll, pin::Pin};

use crate::global::network::post::PostOption;


pub struct ClientHandle{
    pub op: PostOption,
    pub err: i32,
    pub socket: BufReader<TcpStream>,
    pub nreads: u64,
    pub data: String
}
impl Future for ClientHandle{
    type Output = (String, u64);

    fn poll(mut self: Pin<&mut ClientHandle>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let mut buf: Vec<u8> = Vec::new();
        match self.socket.read_until(255, &mut buf){
            Ok(0) => { println!("{} bytes are read", self.nreads); return Poll::Ready((self.data.clone(), self.nreads)) },
            Ok(n) => { self.nreads += n as u64; self.data += &String::from_utf8(buf).unwrap(); return Poll::Pending;},
            Err(e) => { return Poll::Pending; },
        }
    }
}
impl ClientHandle{
    pub fn receive(&mut self) -> (String, u64){
        loop{
            let mut buf: Vec<u8> = Vec::new();
            match self.socket.read_until(255, &mut buf){
                Ok(0) => { println!("{} bytes are read", self.nreads); return (self.data.clone(), self.nreads) },
                Ok(n) => { self.nreads += n as u64; self.data += &String::from_utf8(buf).unwrap(); continue; },
                Err(e) => { return (self.data.clone(), self.nreads)  },
            }
        }
    }
}