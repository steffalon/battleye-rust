use std::net::{Ipv4Addr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use battleye_rust::remote_console::BERemoteConsole;
use battleye_rust::socket::udp::UdpSocketConnection;

#[allow(unused_must_use)]
fn main() {
    let ip = "127.0.0.1".to_string();
    let port = 2306;
    let password = "password".to_string();
    let udp_socket =
        UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).expect("Unable to bind an IP address");
    udp_socket.connect(ip.to_string() + ":" + &port.to_string());

    let be_remote_console: Arc<BERemoteConsole> =
        Arc::new(BERemoteConsole::new(UdpSocketConnection::new(udp_socket)));

    be_remote_console.authenticate(password);

    let keep_alive_socket = be_remote_console.clone();

    thread::spawn(move || loop {
        sleep(Duration::from_secs(35)); // BE recommends sending a keep alive packet before 45 seconds.
        keep_alive_socket.keep_alive();
    });

    thread::spawn(move || loop {
        let response = be_remote_console
            .receive_data()
            .expect("Failed to receive socket data");

        if response.is_empty() {
            continue;
        }

        // println!("{:#04X?}", response);

        match response[1] {
            0x00 => {
                if response[2] == 0x01 {
                    println!("Authentication accepted");
                } else {
                    println!("Password does not match with BattlEye config file");
                }
            }
            0x01 => {
                if response[2] == 0x00 {
                    println!(
                        "{}",
                        String::from_utf8(response[3..response.len()].to_owned()).unwrap()
                    );
                }
            }
            0x02 => {
                println!(
                    "{}",
                    String::from_utf8(response[3..response.len()].to_owned()).unwrap()
                );
            }
            _ => {
                println!("Unknown packet identifier");
            }
        }
    })
    .join();
}
