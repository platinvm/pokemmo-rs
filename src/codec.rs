mod login;

pub use self::login::Login;

pub trait Codec {
    fn encode(&self) -> std::io::Result<Vec<u8>>;
    fn decode(data: &[u8]) -> std::io::Result<Self>
    where
        Self: Sized;
}

/*
todo: implement a macro to reduce boilerplate with this syntax:

#[codec]
pub enum MyCodec {
    VariantA(crate::message::VariantA) = 0x00u8,
    VariantB(crate::message::VariantB) = 0x01u8,
    Unknown{opcode: i8, data: Vec<u8>},
}
*/
