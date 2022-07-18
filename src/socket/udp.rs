use std::io::{Error};
use std::net::{Ipv4Addr, UdpSocket};
use crate::socket::action::SocketAction;

pub struct UdpSocketConnection {
    udp_socket: UdpSocket,
}

impl SocketAction for UdpSocketConnection {
    fn disconnect(&mut self) -> Result<(), Error> {
        self.udp_socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
        Ok(())
    }

    fn listen(&self, buffer_size: usize) -> Result<Vec<u8>, Error> {
        let mut buffer_data = Vec::with_capacity(buffer_size);
        self.udp_socket.recv(&mut buffer_data)?;
        Ok(buffer_data.to_vec())
    }

    fn send(&self, data: &[u8]) -> std::io::Result<usize> {
        self.udp_socket.send(&data)
    }
}

impl UdpSocketConnection {
    pub fn new(udp_socket: UdpSocket) -> Self {
        Self {
            udp_socket
        }
    }
}