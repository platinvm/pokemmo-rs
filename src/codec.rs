mod login;

pub use self::login::Login;

pub trait Codec {
    fn encode(&self) -> std::io::Result<Vec<u8>>;
    fn decode(data: &[u8]) -> std::io::Result<Self>
    where
        Self: Sized;
}

/*
The #[codec] macro reduces boilerplate with this syntax:

Example usage:
```
use pokemmo_codec_macro::codec;

#[codec]
pub enum MyCodec {
    VariantA(crate::message::VariantA) = 0x00u8,
    VariantB(crate::message::VariantB) = 0x01u8,
    Unknown{opcode: u8, data: Vec<u8>},
}
```

This automatically generates:
- The enum definition
- Codec trait implementation (encode/decode methods)
- Into<MyCodec> implementations for each variant type
- TryFrom<MyCodec> implementations for each variant type

See src/codec/login.rs for a real example.
*/
