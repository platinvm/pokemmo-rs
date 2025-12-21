use std::net::TcpStream;

use p256::elliptic_curve::rand_core::OsRng;
use pokemmo_rs::*;

const PRIMARY_OBFUSCATION_VALUE: i64 = 3214621489648854472;
const SECONDARY_OBFUSCATION_VALUE: i64 = -4214651440992349575;
const LOCAL_SERVER: &str = "127.0.0.1:2106";
const REMOTE_SERVER: &str = "loginserver.pokemmo.com:2106";

pub fn main() {
    let tcp_stream = TcpStream::connect(LOCAL_SERVER)
        .or_else(|_| TcpStream::connect(REMOTE_SERVER))
        .unwrap();
    let mut stream = HexLogger::new(tcp_stream);

    let client_hello = ClientHello::new(
        (&() as *const () as usize) as i64,
        std::time::SystemTime::now(),
        PRIMARY_OBFUSCATION_VALUE,
        SECONDARY_OBFUSCATION_VALUE,
    )
    .unwrap();
    stream.write_message(client_hello).unwrap();

    stream.read_message::<ServerHello>().unwrap();

    let client_secret_key = p256::SecretKey::random(&mut OsRng);
    let client_public_key = client_secret_key.public_key();
    let client_ready = ClientReady::new(client_public_key.clone());
    stream.write_message(client_ready).unwrap();
}

use std::io::{self, Read, Write};

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
