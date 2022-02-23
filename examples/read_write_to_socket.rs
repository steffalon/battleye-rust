use std::io::stdin;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

extern crate crc32fast;

use battleye_rust::battleye_rcon::battle_eye_rcon_service::BattlEyeRconService;

fn main() {
    let ip = "127.0.0.1".to_string();
    let port = 2306;
    let password = "password".to_string();

    let battl_eye_rcon_service: Arc<Mutex<BattlEyeRconService>> =
        Arc::new(Mutex::new(BattlEyeRconService::new(ip, port, password)));

    battl_eye_rcon_service.lock().unwrap().prepare_socket();
    battl_eye_rcon_service.lock().unwrap().authenticate();

    let socket_commands = battl_eye_rcon_service.clone();
    let socket_keep_alive = battl_eye_rcon_service.clone();

    thread::spawn(move || loop {
        sleep(Duration::from_secs(35));
        socket_keep_alive.lock().unwrap().keep_alive();
    });

    // A thread for terminal input
    thread::spawn(move || loop {
        let mut input_string = String::new();
        stdin()
            .read_line(&mut input_string)
            .expect("Did not enter a correct string");
        socket_commands
            .lock()
            .unwrap()
            .send_command(input_string.as_str());
    });

    let listen_socket_thread = thread::spawn(move || loop {
        sleep(Duration::from_millis(50)); // Reduce CPU workload
        let response = battl_eye_rcon_service.lock().unwrap().listen();

        if response.is_empty() {
            continue;
        }

        // println!("{:#04X?}", response);

        match response[1] {
            0x00 => {
                if response[2] == 0x01 {
                    println!("Authentication accepted.");
                } else {
                    println!("Password does not match with BattlEye config file");
                }
            }
            0x01 => {
                if response[2] == 0x00 && response.len() > 3 {
                    println!(
                        "{}",
                        String::from_utf8(response[3..response.len()].to_owned()).unwrap()
                    )
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
                println!("Unknown packet identifier.")
            }
        }
    });

    listen_socket_thread.join().unwrap();
}
