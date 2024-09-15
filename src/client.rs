use std::io;

use tokio::net::UdpSocket;

pub struct Client {
    socket: UdpSocket,
}

impl Client {
    pub async fn new() -> io::Result<Self>  {
        let socket = UdpSocket::bind("127.0.0.1:0").await?;
        println!("Client bound to random port");

        Ok(Self { socket })
    }

    pub async fn send_message(&mut self, server_addr: &str, message: &str) -> io::Result<()> {
        self.socket.send_to(message.as_bytes(), server_addr).await?;
        println!("Sent message: {}", message);

        let mut buf = vec![0u8; 1024];

        let (len, addr) = self.socket.recv_from(&mut buf).await?;
        println!("Received response from {}: {}", addr, String::from_utf8_lossy(&buf[..len]));

        Ok(())
    }
}