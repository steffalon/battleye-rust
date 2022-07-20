use crate::remote_console::hash::checksum::crc32::{msg_to_checksum_le_vec, validate};
use crate::remote_console::packet::packet_types;
use crate::socket::action::SocketAction;
use crate::socket::udp::UdpSocketConnection;
use std::io::{Error, ErrorKind};

pub struct BERemoteConsole {
    udp_socket: UdpSocketConnection,
}

impl BERemoteConsole {
    const HEADER_SIZE: usize = 519;

    pub fn new(udp_socket: UdpSocketConnection) -> BERemoteConsole {
        Self { udp_socket }
    }

    pub fn authenticate(&self, password: String) -> std::io::Result<usize> {
        self.send_to_socket(
            packet_types::MESSAGE_TYPE_PACKET_LOGIN,
            password.as_bytes().to_vec(),
        )
    }

    pub fn receive_data(&self) -> Result<Vec<u8>, Error> {
        match self.get_udp_socket().listen(Self::HEADER_SIZE) {
            Ok(mut response) => {
                // Clear unused 0x00 buffers
                for i in 9..response.len() {
                    if response[i].eq(&0x00) {
                        response.truncate(i);
                        break;
                    }
                }

                // Check if CRC-32 server response is valid
                if validate(&response) {
                    let ack = self.acknowledge_msg(response[8]);
                    if ack.is_err() {
                        return Err(ack.err().unwrap());
                    }
                    return Ok(response[6..response.len()].to_vec());
                }

                Err(Error::new(ErrorKind::InvalidData, "Invalid checksum"))
            }
            Err(err) if err.kind() != ErrorKind::WouldBlock => Err(err),
            _ => Ok(vec![]),
        }
    }

    fn acknowledge_msg(&self, sequence: u8) -> std::io::Result<usize> {
        self.send_to_socket(
            packet_types::MESSAGE_TYPE_PACKET_SERVER_MESSAGE,
            [sequence].to_vec(),
        )
    }

    pub fn send_command(&self, command: &str) -> std::io::Result<usize> {
        let mut command_body: Vec<u8> = vec![0];
        command_body.append(&mut command.as_bytes().to_vec());
        self.send_to_socket(packet_types::MESSAGE_TYPE_PACKET_COMMAND, command_body)
    }

    fn send_to_socket(&self, message_type_packet: u8, msg: Vec<u8>) -> std::io::Result<usize> {
        let mut assemble_packets: Vec<u8> = vec![0xFF, message_type_packet];
        assemble_packets.extend(msg);

        let mut crc32check = msg_to_checksum_le_vec(&assemble_packets); // Apply CRC-32 on message

        let mut data = packet_types::STATIC_HEADER.to_vec(); // Start header BE
        data.append(&mut crc32check); // CRC 32 hash
        data.append(&mut assemble_packets); // Regular packet array without CRC 32

        self.get_udp_socket().send(&data)
    }

    fn get_udp_socket(&self) -> &UdpSocketConnection {
        &self.udp_socket
    }

    pub fn keep_alive(&self) -> std::io::Result<usize> {
        self.send_to_socket(packet_types::MESSAGE_TYPE_PACKET_COMMAND, vec![0x00])
    }
}
