use std::{future::Future, net::TcpStream, io::Read, task::Poll, pin::Pin};

use crate::global::network::post::PostOption;


pub struct ClientHandle{
    pub op: PostOption,
    pub err: i32,
    pub socket: TcpStream,
    pub nreads: u64
}
impl Future for ClientHandle{
    type Output = String;

    fn poll(mut self: Pin<&mut ClientHandle>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let mut buf: [u8; 512] = [0; 512];
        loop{
            match self.socket.read(&mut buf){
                Ok(0) => return Poll::Ready(String::from_utf8(buf.to_vec()).unwrap()),
                Ok(n) => self.nreads += n as u64,
                Err(e) => return Poll::Pending,
            }
        }
    }
}