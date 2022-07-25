pub mod crc32 {
    /// Create CRC-32 checksum from an array of bytes.
    pub fn msg_to_checksum_le_vec(msg: &[u8]) -> Vec<u8> {
        crc32fast::hash(msg)
            .to_le_bytes() // Little endian for correct byte sequence
            .to_vec()
    }

    /// Validate an array of bytes using CRC-32. If CRC-32 checksum doesn't match with the CRC-32
    /// message, it will return false.
    pub fn validate(message: &[u8]) -> bool {
        let message_socket_response = &message[6..message.len()];
        if message_socket_response[0] == 0xFF {
            return msg_to_checksum_le_vec(message_socket_response).eq(&message[2..6]);
        }
        false
    }
}

#[cfg(test)]
mod crc_32_test {
    use super::*;

    #[test]
    fn msg_to_checksum() {
        let message: Vec<u8> = vec![0xFF, 0x01, 0x61, 0x62, 0x63, 0x64];
        assert_eq!(crc32::msg_to_checksum_le_vec(&message), [0xD2, 0x31, 0xA0, 0xA4])
    }

    #[test]
    fn validate_msg() {
        let message: Vec<u8> = vec![0x42, 0x45, 0x70, 0x8D, 0x77, 0x62, 0xFF, 0x01, 0x61];
        assert!(crc32::validate(&message));
    }
}
