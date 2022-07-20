pub mod crc32 {
    pub fn msg_to_checksum_le_vec(msg: &[u8]) -> Vec<u8> {
        crc32fast::hash(msg)
            .to_le_bytes() // Little endian for correct byte sequence
            .to_vec()
    }
    pub fn validate(message: &[u8]) -> bool {
        let message_socket_response = &message[6..message.len()];
        if message_socket_response[0] == 0xFF {
            return msg_to_checksum_le_vec(message_socket_response).eq(&message[2..6])
        }
        false
    }
}
