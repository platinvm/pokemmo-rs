use p256::{
    ecdsa::{signature::Signer, SigningKey},
    elliptic_curve::rand_core::OsRng,
    SecretKey,
};
use pokemmo_rs::{
    message::{
        client_hello::ClientHello, client_ready::ClientReady, server_hello::ServerHello,
        ReadMessage, WriteMessage,
    },
    utils::logger::Logger,
};
use std::{
    io::{self, Read, Write},
    net::TcpListener,
};

pub fn main() {
    let listener = TcpListener::bind("127.0.0.1:2106").unwrap();
    println!("[INFO]: Server listening on 127.0.0.1:2106");

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                println!(
                    "[INFO]: New client connection from {}",
                    tcp_stream.peer_addr().unwrap()
                );
                let mut stream = Logger::new(tcp_stream);
                if let Err(e) = handle_client(&mut stream) {
                    eprintln!("[ERROR]: Failed to handle client: {}", e);
                }
            }
            Err(e) => {
                eprintln!("[ERROR]: Failed to accept connection: {}", e);
            }
        }
    }
}

fn handle_client<T: Read + Write>(stream: &mut Logger<T>) -> io::Result<()> {
    let server_secret_key = SecretKey::random(&mut OsRng);
    let server_public_key = server_secret_key.public_key();
    let signing_key = SigningKey::from(&server_secret_key);

    stream.info("Waiting for ClientHello");
    let _client_hello: ClientHello = stream.read_message::<ClientHello>(
        pokemmo_rs::packet::Encryption::None,
        pokemmo_rs::packet::Checksum::None,
    )?;
    stream.info("Received ClientHello");

    let public_key_bytes = server_public_key.to_sec1_bytes();
    let signature = signing_key.sign(&public_key_bytes);
    let server_hello = ServerHello::new(
        server_public_key.clone(),
        signature,
        pokemmo_rs::message::server_hello::Checksum::None,
    );

    stream.info("Sending ServerHello");
    stream.write_message(
        server_hello,
        pokemmo_rs::packet::Encryption::None,
        pokemmo_rs::packet::Checksum::None,
    )?;
    stream.info("Sent ServerHello");

    stream.info("Waiting for ClientReady");
    let _client_ready: ClientReady = stream.read_message::<ClientReady>(
        pokemmo_rs::packet::Encryption::None,
        pokemmo_rs::packet::Checksum::None,
    )?;
    stream.info("Received ClientReady");

    stream.info("Successfully completed handshake with client.");
    Ok(())
}
