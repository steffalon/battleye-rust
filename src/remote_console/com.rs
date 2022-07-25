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

    /// Create new BERemoteConsole including a UDP socket
    pub fn new(udp_socket: UdpSocketConnection) -> BERemoteConsole {
        Self { udp_socket }
    }

    /// Authenticate to BattlEye remote console server. Result will only throw an error if there is
    /// no established connection to the server. Incorrect authentication results an expected
    /// value and therefore will not raise an error.
    pub fn authenticate(&self, password: String) -> std::io::Result<usize> {
        self.send_to_socket(
            packet_types::MESSAGE_TYPE_PACKET_LOGIN,
            password.as_bytes().to_vec(),
        )
    }

    /// Listen to socket and if and only if there is a response from the server, then the
    /// validation can start. Unused bytes gets truncated from buffer.
    ///
    /// After server response validation, gets validation has completed,
    ///
    /// This method relies on an established connection from socket. If the socket is not connected
    /// or lost connection to host, it will fail.
    ///
    /// This method ignores [`ErrorKind::WouldBlock`] and therefore an empty [`Vec`] gets returned.
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

    /// In order to satisfy BattlEye remote console continues session, an acknowledge must be send
    /// after receiving any BattlEye responses.
    ///
    /// To acknowledge a message from the BattlEye server, [`BERemoteConsole::send_to_socket`] is used including the
    /// corresponding packet identifier.
    ///
    /// This method relies on an established connection from socket. If the socket is not connected
    /// or lost connection to host, it will fail.
    ///
    /// Return value is relayed from [`BERemoteConsole::send_to_socket`].
    fn acknowledge_msg(&self, sequence: u8) -> std::io::Result<usize> {
        self.send_to_socket(
            packet_types::MESSAGE_TYPE_PACKET_SERVER_MESSAGE,
            [sequence].to_vec(),
        )
    }

    /// Dispatch a command to server. Command gets composed to make it identifiable to BattlEye.
    ///
    /// This method relies on an established connection from socket. If the socket is not connected
    /// or lost connection to host, it will fail.
    pub fn send_command(&self, command: &str) -> std::io::Result<usize> {
        let mut command_body: Vec<u8> = vec![0];
        command_body.append(&mut command.as_bytes().to_vec());
        self.send_to_socket(packet_types::MESSAGE_TYPE_PACKET_COMMAND, command_body)
    }

    /// Communicate to BattlEye remote console server.
    ///
    /// Packet identifier and message gets merged into single vector containing bytes. Then CRC-32 checksum
    /// gets created and will be included inside the same vector.
    ///
    /// This method relies on an established connection from socket. If the socket is not connected
    /// or lost connection to host, it will fail.
    fn send_to_socket(&self, message_type_packet: u8, msg: Vec<u8>) -> std::io::Result<usize> {
        let mut assemble_packets: Vec<u8> = vec![0xFF, message_type_packet];
        assemble_packets.extend(msg);

        let mut crc32check = msg_to_checksum_le_vec(&assemble_packets); // Apply CRC-32 on message

        let mut data = packet_types::STATIC_HEADER.to_vec(); // Start header BE
        data.append(&mut crc32check); // CRC 32 hash
        data.append(&mut assemble_packets); // Regular packet array without CRC 32

        self.get_udp_socket().send(&data)
    }

    /// Get UDP socket from BERemoteConsole
    fn get_udp_socket(&self) -> &UdpSocketConnection {
        &self.udp_socket
    }

    /// Send keep alive packet to BattlEye remote console server. This will ensure the server to
    /// keep you subscribed.
    ///
    /// This method relies on an established connection from socket. If the socket is not connected
    /// or lost connection to host, it will fail.
    pub fn keep_alive(&self) -> std::io::Result<usize> {
        self.send_to_socket(packet_types::MESSAGE_TYPE_PACKET_COMMAND, vec![0x00])
    }
}
