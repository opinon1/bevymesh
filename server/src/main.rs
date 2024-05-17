use std::{
    io::{self, BufRead},
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};

use netcode::{Client, ClientIndex, ClientState, Server, ServerConfig};

enum Event {
    Connected(ClientIndex),
    Disconnected(ClientIndex),
}

fn main() {
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    let my_secret_private_key = [0; 32];
    let (tx, rx) = mpsc::channel::<Event>();
    let cfg = ServerConfig::with_context(tx.clone())
        .on_connect(move |client_idx, tx| {
            tx.send(Event::Connected(client_idx)).unwrap();
        })
        .on_disconnect(move |client_idx, tx| {
            tx.send(Event::Disconnected(client_idx)).unwrap();
        });
    let mut server =
        Server::with_config("127.0.0.1:12345", 0x11223344, my_secret_private_key, cfg).unwrap();

    let start = Instant::now();
    let tick_rate = Duration::from_secs_f64(1.0 / 60.0);

    let server_thread = thread::spawn(move || loop {
        let now = start.elapsed().as_secs_f64();
        server.update(now);

        while let Some((packet, client_idx)) = server.recv() {
            let s = std::str::from_utf8(&packet).unwrap();
            println!("server received: {s}",);
            server.send(&packet, client_idx).unwrap();
        }
        match rx.try_recv() {
            Ok(Event::Connected(idx)) => {
                log::info!("client {idx} connected");
            }
            Ok(Event::Disconnected(idx)) => {
                log::info!("client {idx} disconnected");
            }
            Err(_) => continue,
        }
        thread::sleep(tick_rate);
    });

    server_thread.join().unwrap();
}
