mod client_hello;
mod client_ready;
mod server_hello;

pub use self::client_hello::ClientHello;
pub use self::client_ready::ClientReady;
pub use self::server_hello::Checksum;
pub use self::server_hello::ServerHello;

pub trait Message: Sized {
    fn serialize(&self) -> std::io::Result<Vec<u8>>;
    fn deserialize(data: &[u8]) -> std::io::Result<Self>;
}

pub use pokemmo_macros::Message;
