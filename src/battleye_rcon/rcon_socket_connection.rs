use std::io::Write;
use std::net::{Ipv4Addr, UdpSocket};
use std::time::Duration;

pub struct BattlEyeRconService {
    ip: String,
    udp_port: i32,
    password: String,
    udp_socket: Option<UdpSocket>,
    sequence_byte: u8,
}

impl BattlEyeRconService {
    const HEADER_SIZE: usize = 10000; // Is this enough?

    pub fn new(ip: String, udp_port: i32, password: String) -> BattlEyeRconService {
        Self {
            ip,
            udp_port,
            password,
            udp_socket: None,
            sequence_byte: 0x00,
        }
    }

    pub fn prepare_socket(&mut self) {
        self.udp_socket = Option::from(
            UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).expect("Unable to bind this address"),
        );
        self.udp_socket
            .as_ref()
            .unwrap()
            .connect(self.ip.to_string() + ":" + &self.udp_port.to_string())
            .expect("Cannot connect to server.");
        self.udp_socket
            .as_ref()
            .unwrap()
            .set_read_timeout(Some(Duration::new(45, 0)))
            .expect("set_read_timeout call failed");
    }

    pub fn authenticate(&mut self) {
        let mut crc32prepare: Vec<u8> = vec![0xFF, 0x00];
        crc32prepare.append(&mut self.password.as_bytes().to_vec());

        let mut crc32checked = crc32fast::hash(&crc32prepare).to_be_bytes().to_vec();
        crc32checked.reverse(); // Reverse CRC-32

        let mut message: Vec<u8> = vec![];
        message.append(&mut [0x42, 0x45].to_vec()); // Start header BE
        message.append(crc32checked.by_ref()); // CRC 32 hash

        message.push(0xFF);
        message.push(0x00); // Login action

        // Payload
        message.append(&mut self.password.as_bytes().to_vec());

        let socket = self.get_udp_socket();
        socket.send(&message).unwrap();

        let response = self.listen();
        // println!("{:#04X?}", response);

        if response[2] == 0x01 {
            println!("Authentication accepted.");
        } else if response[2] == 0x00 {
            println!("Password does not match with BattlEye config file");
        } else {
            println!("Authentication failed");
        }
    }

    pub fn listen(&mut self) -> Vec<u8> {
        let socket = self.get_udp_socket();
        let mut buffer = [0; Self::HEADER_SIZE];

        socket.recv(&mut buffer).unwrap();

        let mut buffer_vec = buffer.to_vec();

        for i in 9..buffer_vec.len() {
            if buffer_vec.get(i).unwrap().eq(&(0x00 as u8)) {
                buffer_vec.truncate(i);
                break;
            }
        }

        // Check if CRC-32 server response is valid
        if Self::is_valid_msg(&buffer_vec) {
            self.acknowledge_msg(buffer_vec[8]);
            return buffer_vec[6..buffer_vec.len()].to_vec();
        }

        // Cannot trust the given response data from remote
        vec![0x00, 0x00, 0x00]
    }

    fn set_sequence(&mut self, sequence: u8) {
        self.sequence_byte = sequence;
    }

    fn acknowledge_msg(&mut self, sequence: u8) {
        let mut crc32prepare: Vec<u8> = vec![0xFF, 0x02];
        crc32prepare.push(sequence);

        let mut crc32checked = crc32fast::hash(&crc32prepare).to_be_bytes().to_vec();
        crc32checked.reverse(); // Reverse CRC-32

        let mut message: Vec<u8> = vec![];
        message.append(&mut [0x42, 0x45].to_vec()); // Start header BE
        message.append(crc32checked.by_ref());

        message.push(0xFF);
        message.push(0x02); // Server message

        message.push(sequence);

        let socket = self.get_udp_socket();
        socket.send(&message).unwrap();
        self.set_sequence(sequence);
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

        println!("Message incorrect");
        false
    }

    pub fn get_udp_port(&self) -> i32 {
        self.udp_port
    }

    fn get_udp_socket(&self) -> &UdpSocket {
        self.udp_socket.as_ref().unwrap()
    }

    fn get_sequence(&self) -> u8 {
        self.sequence_byte
    }

    pub fn send_command(&self, command: &str) {
        let mut message: Vec<u8> = vec![0xFF, 0x01, self.get_sequence()]; // 1-byte sequence number
        message.append(command.as_bytes().to_vec().by_ref()); // Server message (ASCII string without null-terminator)

        let mut crc32check = crc32fast::hash(&message.clone()).to_be_bytes().to_vec();
        crc32check.reverse(); // Reverse CRC-32

        let mut header_payload = [0x42, 0x45].to_vec(); // Start header BE
        header_payload.append(crc32check.by_ref()); // CRC 32 hash
        header_payload.append(&mut message); // Regular bytes array in correct sequence.

        self.get_udp_socket().send(&header_payload).unwrap();
    }

    fn send_command_as_bytes(&self, mut command: Vec<u8>) {
        let mut message: Vec<u8> = vec![0xFF, 0x01, self.get_sequence()]; // 1-byte sequence number
        message.append(&mut command); // Server message (ASCII string without null-terminator)

        let mut crc32check = crc32fast::hash(&message.clone()).to_be_bytes().to_vec();
        crc32check.reverse(); // Reverse CRC-32

        let mut header_payload = [0x42, 0x45].to_vec(); // Start header BE
        header_payload.append(crc32check.by_ref()); // CRC 32 hash
        header_payload.append(&mut message); // Regular bytes array in correct sequence.

        self.get_udp_socket().send(&header_payload).unwrap();
    }

    pub fn keep_alive(&self) {
        self.send_command_as_bytes(vec![0x00, 0x00]);
    }
}
