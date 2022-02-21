use std::io::Write;
use std::net::{Ipv4Addr, UdpSocket};

pub struct BattlEyeRconService {
    ip: String,
    udp_port: i32,
    password: String,
    udp_socket: Option<UdpSocket>,
    sequence_byte: u8,
}

impl BattlEyeRconService {
    const HEADER_SIZE: usize = 10000; // Is this enough? Time will tell.

    pub fn new(ip: String, udp_port: i32, password: String) -> Self {
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
            .expect("Cannot connect to server.")
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

        // Cannot trust given response data from the server
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

    // pub fn dispatch_data(buf: Vec<usize>) {}
}
