use std::io;

use tokio::net::UdpSocket;

pub struct Server {
    pub socket: UdpSocket,
    pub addr: String,
}

impl Server {
    pub async fn new(addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        println!("Server is listening on {:?}", addr);

        Ok(Self { socket, addr: addr.to_string() })
    }

    pub async fn run(&mut self) -> io::Result<()> {
        let mut buf = vec![0u8; 1024]; 

        loop {
            let (len, addr) = self.socket.recv_from(&mut buf).await?;
            println!("Received {} bytes from {}", len, addr);

            self.socket.send_to(&buf[..len], &addr).await?;
        }
    }
}