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
The Message derive macro has been implemented and can be used with this syntax:

#[derive(Message)]
pub struct MyMessage {
    field1: u32,
    field2: i64,
    #[prefixed(i16)] // required for Vec and String fields
    field3: Vec<u8>,
}

The macro automatically implements the Message trait with serialize() and deserialize() methods.
Supported types:
- Integer types: i8, i16, i32, i64, u8, u16, u32, u64 (serialized as little-endian)
- Vec<u8> with #[prefixed(T)] attribute where T is an integer type for the length prefix
- String with #[prefixed(T)] attribute (not yet implemented but can be added)

Example usage is available in pokemmo-macros/tests/macro_tests.rs
*/
