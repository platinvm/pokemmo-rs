use p256::elliptic_curve::rand_core::OsRng;
use p256::ecdsa::{signature::Signer, SigningKey};
use pokemmo::{
    codec::Login,
    context::WithContext,
    message::{Checksum, ClientHello, ClientReady, ServerHello},
};
use std::net::TcpListener;
use std::io::Write;

const PRIMARY_OBFUSCATION_VALUE: i64 = 3214621489648854472;
const SECONDARY_OBFUSCATION_VALUE: i64 = -4214651440992349575;
const BIND_ADDR: &str = "127.0.0.1:2106";

pub fn main() {
    let listener = TcpListener::bind(BIND_ADDR).expect("Failed to bind server socket");
    println!("Server listening on {}", BIND_ADDR);

    // Accept a single client for this example
    let (stream, addr) = listener.accept().expect("Failed to accept connection");
    println!("Accepted connection from {}", addr);

    let mut stream = stream.with_context::<Login>();

    // Read the initial ClientHello
    let client_hello: ClientHello = stream
        .read_message::<ClientHello>()
        .expect("Failed to read ClientHello");

    // Optionally de-obfuscate and log
    let integrity = client_hello.integrity(PRIMARY_OBFUSCATION_VALUE);
    let timestamp = client_hello
        .timestamp(PRIMARY_OBFUSCATION_VALUE, SECONDARY_OBFUSCATION_VALUE)
        .expect("Failed to decode client timestamp");
    println!(
        "ClientHello: integrity={}, timestamp={:?}",
        integrity, timestamp
    );

    // Prepare a server key and signature
    let signing_key = SigningKey::random(&mut OsRng);
    let server_public_key: p256::PublicKey = signing_key.verifying_key().into();

    // Sign the server's public key bytes for demonstration
    let pubkey_bytes = server_public_key.to_sec1_bytes();
    let signature = signing_key.sign(&pubkey_bytes);

    // Build ServerHello; choose a simple checksum policy
    let server_hello = ServerHello::new(server_public_key, signature, Checksum::None);

    // Send ServerHello back to the client
    stream
        .write_message(server_hello)
        .expect("Failed to write ServerHello");
    println!("Sent ServerHello");

    // Read ClientReady from the client
    let client_ready: ClientReady = stream
        .read_message::<ClientReady>()
        .expect("Failed to read ClientReady");

    let client_pub = client_ready
        .public_key()
        .expect("Failed to parse client public key");
    println!(
        "ClientReady received: client pubkey (SEC1) size={} bytes",
        client_pub.to_sec1_bytes().len()
    );

    // Example complete; close connection
    let _ = stream.flush();
    println!("Session complete; closing connection");
}
