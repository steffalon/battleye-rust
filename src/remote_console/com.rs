use std::io::{Error, ErrorKind, Write};
use crate::remote_console::hash::crc32;
use crate::socket::udp::UdpSocketConnection;
use crate::remote_console::packet::packet_types;
use crate::socket::action::SocketAction;

pub struct BERemoteConsole {
    udp_socket: UdpSocketConnection,
}

impl BERemoteConsole {
    const HEADER_SIZE: usize = 10000;

    pub fn new(udp_socket: UdpSocketConnection) -> BERemoteConsole {
        Self {
            udp_socket,
        }
    }

    pub fn authenticate(&mut self, password: String) -> std::io::Result<usize> {
        self.send_to_socket(
            packet_types::MESSAGE_TYPE_PACKET_LOGIN,
            password.as_bytes().to_vec(),
        )
    }

    pub fn receive_data(&mut self) -> Result<Vec<u8>, Error> {
        match self.get_udp_socket().listen(Self::HEADER_SIZE) {
            Ok(mut response) => {
                for i in 9..response.len() {
                    if response.get(i).unwrap().eq(&(0x00 as u8)) {
                        response.truncate(i);
                        break;
                    }
                }

                // Check if CRC-32 server response is valid
                if crc32::validate(&response) {
                    let ack = self.acknowledge_msg(response[8]);
                    if ack.is_err() {
                        return Err(ack.err().unwrap())
                    }
                    return Ok(response[6..response.len()].to_vec());
                }

                Err(Error::new(ErrorKind::InvalidData, "Invalid checksum"))
            }
            Err(err) if err.kind() != ErrorKind::WouldBlock => {
                Err(err)
            }
            _ => {
                Ok(vec![])
            }
        }
    }

    fn acknowledge_msg(&mut self, sequence: u8) -> std::io::Result<usize> {
        self.send_to_socket(
            packet_types::MESSAGE_TYPE_PACKET_SERVER_MESSAGE,
            [sequence].to_vec(),
        )
    }

    pub fn send_command(&mut self, command: &str) -> std::io::Result<usize> {
        let mut command_body: Vec<u8> = vec![0];
        command_body.append(&mut command.as_bytes().to_vec());
        self.send_to_socket(packet_types::MESSAGE_TYPE_PACKET_COMMAND, command_body)
    }

    fn send_to_socket(&mut self, message_type_packet: u8, mut msg: Vec<u8>) -> std::io::Result<usize> {
        let mut assemble_packets: Vec<u8> = vec![0xFF, message_type_packet];
        assemble_packets.append(msg.by_ref());

        // Create CRC 32 hash from msg packet array
        let mut crc32check = crc32fast::hash(&assemble_packets.clone())
            .to_be_bytes()
            .to_vec();
        crc32check.reverse(); // Reverse CRC-32

        let mut data = packet_types::STATIC_HEADER.to_vec(); // Start header BE
        data.append(crc32check.by_ref()); // CRC 32 hash
        data.append(&mut assemble_packets); // Regular packet array without CRC 32

        self.get_udp_socket().send(&data)
    }

    fn get_udp_socket(&self) -> &UdpSocketConnection {
        &self.udp_socket
    }

    pub fn keep_alive(&mut self) -> std::io::Result<usize> {
        self.send_to_socket(packet_types::MESSAGE_TYPE_PACKET_COMMAND, vec![0x00])
    }
}