use std::net::TcpStream;

use pokemmo_rs::prelude::*;

pub fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:2106").unwrap();
    let mut context = Context::default();

    let client_hello = ClientHello::default();

    println!("[INFO]: Sending ClientHello");
    stream.write_packet(&client_hello, &context).unwrap();
    println!("[INFO]: Sent ClientHello");

    println!("[INFO]: Receiving ServerHello");
    let server_hello: ServerHello = stream.read_packet(&context).unwrap();
    println!("[INFO]: Received ServerHello");

    context.server_public_key = Some(server_hello.public_key().clone());
    context.server_signature = Some(server_hello.signature().clone());
    context.checksum_size = Some(server_hello.checksum_size());

    // Generate client's key pair
    let client_secret_key = p256::SecretKey::random(&mut rand::thread_rng());
    let client_public_key = client_secret_key.public_key();
    let client_ready = ClientReady::new(client_public_key.clone());

    println!("[INFO]: Sending ClientReady");
    stream.write_packet(&client_ready, &context).unwrap();
    println!("[INFO]: Sent ClientReady");

    context.client_public_key = Some(client_public_key);
    context.client_secret_key = Some(client_secret_key);
    println!("[INFO]: Successfully completed handshake with server.");

    if let Some(ref public_key) = context.server_public_key {
        let bytes = public_key.to_sec1_bytes();
        println!("[INFO]: Server Public Key ({} bytes):", bytes.len());
        println!("{}", format_hex(&bytes));
    }

    if let Some(ref signature) = context.server_signature {
        let bytes = signature.to_bytes();
        println!("[INFO]: Server Signature ({} bytes):", bytes.len());
        println!("{}", format_hex(&bytes));
    }

    if let Some(checksum_size) = context.checksum_size {
        println!("[INFO]: Checksum Size: {} bytes", checksum_size);
    }

    if let Some(ref secret_key) = context.client_secret_key {
        let bytes = secret_key.to_bytes();
        println!("[INFO]: Client Secret Key ({} bytes):", bytes.len());
        println!("{}", format_hex(&bytes));
    }
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
