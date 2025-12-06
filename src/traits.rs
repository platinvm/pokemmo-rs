use crate::Context;

/// A Payload is the data portion of a packet without an opcode prefix.
///
/// The Payload trait defines methods for encoding and decoding the data,
/// while the Packet trait handles the opcode prefix.
pub trait Payload: Sized {
    const OPCODE: i8;

    fn encode(&self, data: impl std::io::Write, ctx: &Context) -> Result<(), std::io::Error>;
    fn decode(data: impl std::io::Read, ctx: &Context) -> Result<Self, std::io::Error>;
}

/// A Packet is a Payload with an opcode prefix.
///
/// The Packet trait provides default implementations for encoding and decoding
/// that handle the opcode prefix automatically.
pub trait Packet<P: Payload>: Sized {
    fn encode(&self, data: impl std::io::Write, ctx: &Context) -> Result<(), std::io::Error>;
    fn decode(data: impl std::io::Read, ctx: &Context) -> Result<Self, std::io::Error>;
}

impl<P: Payload> Packet<P> for P {
    fn encode(&self, mut data: impl std::io::Write, ctx: &Context) -> Result<(), std::io::Error> {
        data.write_all(&[P::OPCODE as u8])?;
        Payload::encode(self, data, ctx)
    }

    fn decode(mut data: impl std::io::Read, ctx: &Context) -> Result<Self, std::io::Error> {
        let mut opcode_buf = [0u8; 1];
        data.read_exact(&mut opcode_buf)?;
        let opcode = opcode_buf[0] as i8;
        if opcode != P::OPCODE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Expected opcode {}, found {}", P::OPCODE, opcode),
            ));
        }

        P::decode(data, ctx)
    }
}
