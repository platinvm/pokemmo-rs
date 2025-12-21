use std::io::{self, Read, Write};
use std::net::TcpListener;

use p256::ecdsa::{signature::Signer, SigningKey};
use p256::elliptic_curve::rand_core::OsRng;
use p256::SecretKey;
use pokemmo_rs::*;

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
                let mut stream = HexLogger::new(tcp_stream);
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

fn handle_client<T: Read + Write>(stream: &mut HexLogger<T>) -> io::Result<()> {
    // Generate server's signing key pair
    let server_secret_key = SecretKey::random(&mut OsRng);
    let server_public_key = server_secret_key.public_key();
    let signing_key = SigningKey::from(&server_secret_key);

    stream.info("Waiting for ClientHello");
    let _client_hello: ClientHello = stream.read_message()?;
    stream.info("Received ClientHello");

    // Sign the server's public key to create the signature
    let public_key_bytes = server_public_key.to_sec1_bytes();
    let signature = signing_key.sign(&public_key_bytes);
    let server_hello = ServerHello::new(
        server_public_key.clone(),
        signature,
        Checksum::HmacSha256(16),
    );

    stream.info("Sending ServerHello");
    stream.write_message(server_hello)?;
    stream.info("Sent ServerHello");

    stream.info("Waiting for ClientReady");
    let _client_ready: ClientReady = stream.read_message()?;
    stream.info("Received ClientReady");

    stream.info("Successfully completed handshake with client.");
    Ok(())
}

struct HexLogger<T: Read + Write> {
    inner: T,
}

impl<T: Read + Write> HexLogger<T> {
    pub fn new(inner: T) -> Self {
        HexLogger { inner }
    }
    fn hexdump(prefix: &str, buf: &[u8]) {
        print!("{} ({} bytes):\n", prefix, buf.len());
        for (i, chunk) in buf.chunks(16).enumerate() {
            print!("{:04x}  ", i * 16);
            for (j, byte) in chunk.iter().enumerate() {
                if j == 8 {
                    print!(" ");
                }
                print!("{:02x} ", byte);
            }
            for j in chunk.len()..16 {
                if j == 8 {
                    print!(" ");
                }
                print!("   ");
            }
            print!(" |");
            for byte in chunk {
                let ch = if *byte >= 0x20 && *byte <= 0x7e {
                    *byte as char
                } else {
                    '.'
                };
                print!("{}", ch);
            }
            println!("|");
        }
    }
    pub fn info(&self, msg: &str) {
        println!("[INFO]: {}", msg);
    }
}

impl<T: Read + Write> Read for HexLogger<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        Self::hexdump("Read", &buf[..n]);
        Ok(n)
    }
}

impl<T: Read + Write> Write for HexLogger<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        Self::hexdump("Write", &buf[..n]);
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        let res = self.inner.flush();
        println!("Flush");
        res
    }
}
