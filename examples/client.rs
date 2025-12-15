use std::net::TcpStream;

use p256::elliptic_curve::rand_core::OsRng;
use pokemmo_rs::prelude::*;

const EXTERNAL_SERVER: &str = "loginserver.pokemmo.com:2106";
const INTERNAL_SERVER: &str = "127.0.0.1:2106";

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let use_internal = args
        .get(1)
        .map(|s| s == "--internal" || s == "-i")
        .unwrap_or(false);

    let server = if use_internal {
        INTERNAL_SERVER
    } else {
        EXTERNAL_SERVER
    };
    println!("[INFO]: Connecting to {}", server);

    let mut stream = TcpStream::connect(server).unwrap();

    let client_hello = ClientHello::default();
    let client_hello_ctx = ClientHelloContext::default();

    println!("[INFO]: Sending ClientHello");
    stream
        .write_packet(&client_hello, &client_hello_ctx)
        .unwrap();
    println!("[INFO]: Sent ClientHello");

    println!("[INFO]: Receiving ServerHello");
    let server_hello: ServerHello = stream.read_packet(&()).unwrap();
    println!("[INFO]: Received ServerHello");

    // Generate client's key pair
    let client_secret_key = p256::SecretKey::random(&mut OsRng);
    let client_public_key = client_secret_key.public_key();
    let client_ready = ClientReady::new(client_public_key.clone());

    println!("[INFO]: Sending ClientReady");
    stream.write_packet(&client_ready, &()).unwrap();
    println!("[INFO]: Sent ClientReady");

    println!("[INFO]: Successfully completed handshake with server.");

    let public_key = server_hello.public_key();
    let bytes = public_key.to_sec1_bytes();
    println!("[INFO]: Server Public Key ({} bytes):", bytes.len());
    println!("{}", format_hex(&bytes));

    let signature = server_hello.signature();
    let bytes = signature.to_bytes();
    println!("[INFO]: Server Signature ({} bytes):", bytes.len());
    println!("{}", format_hex(&bytes));

    let checksum_size = server_hello.checksum_size();
    println!("[INFO]: Checksum Size: {} bytes", checksum_size);

    let bytes = client_secret_key.to_bytes();
    println!("[INFO]: Client Secret Key ({} bytes):", bytes.len());
    println!("{}", format_hex(&bytes));
}

fn format_hex(data: &[u8]) -> String {
    let mut result = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        result.push_str(&format!("{:04x}  ", i * 16));

        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                result.push(' ');
            }
            result.push_str(&format!("{:02x} ", byte));
        }

        // Padding for incomplete lines
        for j in chunk.len()..16 {
            if j == 8 {
                result.push(' ');
            }
            result.push_str("   ");
        }

        // ASCII representation
        result.push_str(" |");
        for byte in chunk {
            let ch = if *byte >= 0x20 && *byte <= 0x7e {
                *byte as char
            } else {
                '.'
            };
            result.push(ch);
        }
        result.push('|');

        if i < data.chunks(16).len() - 1 {
            result.push('\n');
        }
    }
    result
}
