mod login;

pub use self::login::Login;
pub use pokemmo_macros::codec;

pub trait Codec {
    fn encode(&self) -> std::io::Result<Vec<u8>>;
    fn decode(data: &[u8]) -> std::io::Result<Self>
    where
        Self: Sized;
}
