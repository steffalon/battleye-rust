use std::io::{ErrorKind, Write, Error};
use std::net::{Ipv4Addr, UdpSocket};

pub struct BattlEyeRconService {
    ip: String,
    udp_port: u16,
    password: String,
    udp_socket: Option<UdpSocket>,
}

impl BattlEyeRconService {
    const HEADER_SIZE: usize = 10000;
    // Is this enough?
    const STATIC_HEADER: [u8; 2] = [0x42, 0x45]; // Required identifier

    // Constants of packet types for command purpose
    const MESSAGE_TYPE_PACKET_LOGIN: u8 = 0x00;
    const MESSAGE_TYPE_PACKET_COMMAND: u8 = 0x01;
    const MESSAGE_TYPE_PACKET_SERVER_MESSAGE: u8 = 0x02; // Also required for acknowledging packets from remote

    pub fn new(ip: String, udp_port: u16, password: String) -> BattlEyeRconService {
        Self {
            ip,
            udp_port,
            password,
            udp_socket: None,
        }
    }

    pub fn prepare_socket(&mut self) -> Result<(), Error> {
        let udp_socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
        udp_socket.connect(self.ip.to_string() + ":" + &self.udp_port.to_string())?;
        udp_socket.set_nonblocking(true)?;

        self.udp_socket = Option::from(udp_socket);
        Ok(())
    }

    pub fn authenticate(&mut self) -> std::io::Result<usize> {
        self.send_to_socket(
            Self::MESSAGE_TYPE_PACKET_LOGIN,
            self.password.as_bytes().to_vec(),
        )
    }

    pub fn listen(&mut self) -> Result<Vec<u8>, Error> {
        let mut buffer = [0; Self::HEADER_SIZE];

        match self.get_udp_socket().recv(&mut buffer) {
            Ok(..) => {
                let mut buffer_vec = buffer.to_vec();

                for i in 9..buffer_vec.len() {
                    if buffer_vec.get(i).unwrap().eq(&(0x00 as u8)) {
                        buffer_vec.truncate(i);
                        break;
                    }
                }

                // Check if CRC-32 server response is valid
                if Self::is_valid_msg(&buffer_vec) {
                    let ack = self.acknowledge_msg(buffer_vec[8]);
                    if ack.is_err() {
                        return Err(ack.err().unwrap())
                    }
                    return Ok(buffer_vec[6..buffer_vec.len()].to_vec());
                }

                Ok(vec![])
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
            Self::MESSAGE_TYPE_PACKET_SERVER_MESSAGE,
            [sequence].to_vec(),
        )
    }

    fn send_to_socket(&mut self, message_type_packet: u8, mut msg: Vec<u8>) -> std::io::Result<usize> {
        let mut assemble_packets: Vec<u8> = vec![0xFF, message_type_packet];
        assemble_packets.append(msg.by_ref());

        // Create CRC 32 hash from msg packet array
        let mut crc32check = crc32fast::hash(&assemble_packets.clone())
            .to_be_bytes()
            .to_vec();
        crc32check.reverse(); // Reverse CRC-32

        let mut data = Self::STATIC_HEADER.to_vec(); // Start header BE
        data.append(crc32check.by_ref()); // CRC 32 hash
        data.append(&mut assemble_packets); // Regular packet array without CRC 32

        self.get_udp_socket().send(&data)
    }

    fn is_valid_msg(message: &Vec<u8>) -> bool {
        let message_socket_response = &message[6..message.len()];
        if message_socket_response[0] == 0xFF {
            let mut crc32check = crc32fast::hash(&message_socket_response)
                .to_be_bytes()
                .to_vec();

            crc32check.reverse();

            if crc32check.eq(&message[2..6]) {
                return true;
            }
        }
        false
    }

    pub fn get_udp_port(&self) -> u16 {
        self.udp_port
    }

    fn get_udp_socket(&self) -> &UdpSocket {
        self.udp_socket.as_ref().unwrap()
    }

    pub fn send_command(&mut self, command: &str) -> std::io::Result<usize> {
        let mut command_body: Vec<u8> = vec![0];
        command_body.append(&mut command.as_bytes().to_vec());
        self.send_to_socket(Self::MESSAGE_TYPE_PACKET_COMMAND, command_body)
    }

    pub fn keep_alive(&mut self) -> std::io::Result<usize> {
        self.send_to_socket(Self::MESSAGE_TYPE_PACKET_COMMAND, vec![0x00])
    }
}
