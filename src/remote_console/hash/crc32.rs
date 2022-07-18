pub fn validate(message: &Vec<u8>) -> bool {
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