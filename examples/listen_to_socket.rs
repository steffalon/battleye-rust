use std::thread;

extern crate crc32fast;

use battleye_rust::battleye_rcon::rcon_socket_connection::BattlEyeRconService;

fn main() {
    let mut battl_eye_rcon_service: BattlEyeRconService =
        BattlEyeRconService::new("127.0.0.1".to_string(), 2306, String::from("my_pasword"));
    battl_eye_rcon_service.prepare_socket();
    battl_eye_rcon_service.authenticate();

    let mut did_send = false;

    let listen_socket_thread = thread::spawn(move || loop {
        let response = battl_eye_rcon_service.listen();
        // println!("{:#04X?}", response);

        match response[1] {
            0x01 => {
                if response[2] == 0x00 {
                    continue;
                }
            }
            0x02 => {
                println!(
                    "{}",
                    String::from_utf8(response[3..response.len()].to_owned()).unwrap()
                );
                if !did_send {
                    battl_eye_rcon_service.send_command("say -1 hello");
                    did_send = true;
                }
            }
            _ => {
                println!("Unknown packet identifier.")
            }
        }
    });

    // thread::spawn(move || loop {
    //     thread::sleep(Duration::from_millis(25000));
    //     BATTLEYE_SERVICE.keep_alive();
    // });

    listen_socket_thread.join().unwrap();
}
