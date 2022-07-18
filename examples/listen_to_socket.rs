use std::net::{Ipv4Addr, UdpSocket};
use std::sync::{Arc, Mutex};
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
    let udp_socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).expect("");
    udp_socket.connect(ip.to_string() + ":" + &port.to_string());
    udp_socket.set_nonblocking(true);

    let be_remote_console: Arc<Mutex<BERemoteConsole>> =
        Arc::new(
            Mutex::new(
                BERemoteConsole::new(
                    UdpSocketConnection::new(udp_socket)
                )
            )
        );

    {
        let mut lock = be_remote_console.lock().unwrap();
        lock.prepare_socket().expect("Initiating socket failed");
        lock.authenticate(password);
    }

    let keep_alive_socket = be_remote_console.clone();

    thread::spawn(move || loop {
        sleep(Duration::from_secs(35)); // BE recommends sending a keep alive packet before 45 seconds.
        keep_alive_socket.lock().unwrap().keep_alive();
    });

    thread::spawn(move || loop {
        sleep(Duration::from_millis(50)); // Reduce CPU workload
        let response = be_remote_console
            .lock()
            .unwrap()
            .listen()
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
                } else {
                    continue;
                }
            }
            0x02 => {
                println!(
                    "{}",
                    String::from_utf8(response[3..response.len()].to_owned()).unwrap()
                )
            }
            _ => {
                println!("Unknown packet identifier")
            }
        }
    }).join();
}
