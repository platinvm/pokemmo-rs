use p256::elliptic_curve::rand_core::OsRng;
use pokemmo_rs::{
    message::{ClientHello, ClientReady, ReadMessage, ServerHello, WriteMessage},
    utils::logger::Logger,
};
use std::net::TcpStream;

const PRIMARY_OBFUSCATION_VALUE: i64 = 3214621489648854472;
const SECONDARY_OBFUSCATION_VALUE: i64 = -4214651440992349575;
const LOCAL_SERVER: &str = "127.0.0.1:2106";
const REMOTE_SERVER: &str = "loginserver.pokemmo.com:2106";

pub fn main() {
    let stream = TcpStream::connect(LOCAL_SERVER)
        .or_else(|_| TcpStream::connect(REMOTE_SERVER))
        .unwrap();

    let mut stream = Logger::new(stream);

    let client_hello = ClientHello::new(
        (&() as *const () as usize) as i64,
        std::time::SystemTime::now(),
        PRIMARY_OBFUSCATION_VALUE,
        SECONDARY_OBFUSCATION_VALUE,
    )
    .unwrap();

    stream
        .write_message(
            client_hello,
            pokemmo_rs::packet::Encryption::None,
            pokemmo_rs::packet::Checksum::None,
        )
        .unwrap();

    stream
        .read_message::<ServerHello>(
            pokemmo_rs::packet::Encryption::None,
            pokemmo_rs::packet::Checksum::None,
        )
        .unwrap();

    let client_secret_key = p256::SecretKey::random(&mut OsRng);
    let client_public_key = client_secret_key.public_key();
    let client_ready = ClientReady::new(client_public_key.clone());

    stream
        .write_message(
            client_ready,
            pokemmo_rs::packet::Encryption::None,
            pokemmo_rs::packet::Checksum::None,
        )
        .unwrap();
}
