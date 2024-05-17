use std::{
    io::{self, BufRead},
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};

use rand::Rng;

use netcode::{Client, ClientState, ConnectToken};

fn main() {
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    let mut rng = rand::thread_rng();

    let protocol_id = 0x11223344;
    let private_key = [0; 32]; // you can also provide your own key
    let client_id: u64 = rng.gen(); // globally unique identifier for an authenticated client
    let server_address = "127.0.0.1:12345"; // the server's public address (can also be multiple addresses)
    let connect_token = ConnectToken::build(server_address, protocol_id, client_id, private_key)
        .generate()
        .unwrap();
    let buf = connect_token.try_into_bytes().unwrap();

    let start = Instant::now();
    let tick_rate = Duration::from_secs_f64(1.0 / 60.0);

    let mut client = Client::new(&buf).unwrap();
    client.connect();

    let (tx, rx) = mpsc::channel::<String>();
    let client_thread = thread::spawn(move || loop {
        let now = start.elapsed().as_secs_f64();
        client.update(now);

        if let Some(packet) = client.recv() {
            println!("echoed back: {}", std::str::from_utf8(&packet).unwrap());
        }
        if let ClientState::Connected = client.state() {
            match rx.recv_timeout(tick_rate) {
                Ok(msg) if msg == "q" => {
                    client.disconnect().unwrap();
                    break;
                }
                Ok(msg) => {
                    if !msg.is_empty() {
                        client.send(msg.as_bytes()).unwrap();
                    }
                }
                Err(RecvTimeoutError::Timeout) => continue,
                Err(_) => break,
            }
        }
        thread::sleep(tick_rate);
    });

    for line in io::stdin().lock().lines() {
        let input = line.unwrap();
        tx.send(input.clone()).unwrap();
        if input == "q" {
            break;
        }
    }

    client_thread.join().unwrap();
}
