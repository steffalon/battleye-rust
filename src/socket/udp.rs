use crate::socket::action::SocketAction;
use std::io::Error;
use std::net::{Ipv4Addr, UdpSocket};

pub struct UdpSocketConnection {
    udp_socket: UdpSocket,
}

impl SocketAction for UdpSocketConnection {
    /// Disconnect socket
    fn disconnect(&mut self) -> Result<(), Error> {
        self.udp_socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
        Ok(())
    }

    /// Listen to socket. It will fail if socket is not connector or disconnected.
    fn listen(&self, buffer_size: usize) -> Result<Vec<u8>, Error> {
        let mut buffer_data = vec![0; buffer_size];
        self.udp_socket.recv(&mut buffer_data)?;
        Ok(buffer_data.to_vec())
    }

    /// Send bytes to server
    fn send(&self, data: &[u8]) -> std::io::Result<usize> {
        self.udp_socket.send(data)
    }
}

impl UdpSocketConnection {
    /// Create new UdpSocketConnection
    pub fn new(udp_socket: UdpSocket) -> Self {
        Self { udp_socket }
    }
}
