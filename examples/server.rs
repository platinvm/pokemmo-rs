use std::net::{TcpListener, TcpStream};

use p256::ecdsa::{signature::Signer, SigningKey};
use p256::SecretKey;
use pokemmo_rs::prelude::*;

fn handle_client(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut context = Context::default();

    // Generate server's signing key pair
    let server_secret_key = SecretKey::random(&mut rand::thread_rng());
    let server_public_key = server_secret_key.public_key();
    let signing_key = SigningKey::from(&server_secret_key);

    println!("[INFO]: Waiting for ClientHello");
    let _client_hello: ClientHello = stream.read_packet(&context)?;
    println!("[INFO]: Received ClientHello");

    // Sign the server's public key to create the signature
    // TODO: In a real implementation, sign with a long-term CA key
    let public_key_bytes = server_public_key.to_sec1_bytes();
    let signature = signing_key.sign(&public_key_bytes);

    let server_hello = ServerHello::new(server_public_key.clone(), signature, 16);

    println!("[INFO]: Sending ServerHello");
    stream.write_packet(&server_hello, &context)?;
    println!("[INFO]: Sent ServerHello");

    context.server_public_key = Some(server_public_key);
    context.server_secret_key = Some(server_secret_key);

    println!("[INFO]: Waiting for ClientReady");
    let _client_ready: ClientReady = stream.read_packet(&context)?;
    println!("[INFO]: Received ClientReady");

    println!("[INFO]: Successfully completed handshake with client.");

    Ok(())
}

pub fn main() {
    let listener = TcpListener::bind("127.0.0.1:2106").unwrap();
    println!("[INFO]: Server listening on 127.0.0.1:2106");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!(
                    "[INFO]: New client connection from {}",
                    stream.peer_addr().unwrap()
                );
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
