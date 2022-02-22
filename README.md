![logo](https://i.imgur.com/jPesxDd.png)

# BattlEye Remote Control Rust

It is a modest BattlEye RCON library made in rust. This support developers to perform authentication, 
acknowledging packets and sending commands via UDP socket connection. There is an example how you can 
implement/use this library.

# Features

- [x] Authentication
- [x] Dispatch commands
- [x] Observe packets
- [x] Acknowledge logic after receiving a packet from the server
- [x] CRC-32 validation on every received packet
- [ ] Keep alive of connection (Within 45 seconds send an empty 2-byte command packet)
- [ ] Non-blocking read & write to socket

## Cargo dependencies

- crc32fast