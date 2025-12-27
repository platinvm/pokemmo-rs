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

/*
todo: implement a macro to reduce boilerplate with this syntax:

#[derive(Message)]
pub struct MyMessage {
    field1: u32,
    field2: i64,
    #[prefixed(i16)] // required for Vec and String fields
    field3: Vec<u8>,
}
*/
