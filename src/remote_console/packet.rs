pub mod packet_types {
    // Is this enough?
    pub const STATIC_HEADER: [u8; 2] = [0x42, 0x45]; // Required identifier ['B','E']

    // Constants of packet types for command purpose
    pub const MESSAGE_TYPE_PACKET_LOGIN: u8 = 0x00;
    pub const MESSAGE_TYPE_PACKET_COMMAND: u8 = 0x01;
    pub const MESSAGE_TYPE_PACKET_SERVER_MESSAGE: u8 = 0x02; // Also required for acknowledging packets from remote
}