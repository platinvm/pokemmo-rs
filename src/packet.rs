use crate::payload;

pub trait Packet<P: payload::Payload>: Sized {
    fn encode_packet(&self, ctx: &P::Context) -> Result<Vec<u8>, P::Error>;
    fn decode_packet(data: &[u8], ctx: &P::Context) -> Result<Self, P::Error>;
}

impl<P: payload::Payload> Packet<P> for P {
    fn encode_packet(&self, ctx: &P::Context) -> Result<Vec<u8>, P::Error> {
        self.serialize(ctx)
    }

    fn decode_packet(data: &[u8], ctx: &P::Context) -> Result<Self, P::Error> {
        P::deserialize(data, ctx)
    }
}

pub mod ext {
    use crate::payload;

    pub trait WritePacket: std::io::Write {
        fn write_packet<P: payload::Payload<Error = std::io::Error>>(
            &mut self,
            packet: &P,
            ctx: &P::Context,
        ) -> Result<(), std::io::Error> {
            let payload_buf = packet.serialize(ctx)?;

            let size = payload_buf.len() as i16 + 3;
            self.write_all(&size.to_le_bytes())?;
            self.write_all(&[P::OPCODE as u8])?;
            self.write_all(&payload_buf)?;

            self.flush()?;

            Ok(())
        }
    }

    impl<T: std::io::Write> WritePacket for T {}

    pub trait ReadPacket: std::io::Read {
        fn read_packet<P: payload::Payload<Error = std::io::Error>>(
            &mut self,
            ctx: &P::Context,
        ) -> Result<P, std::io::Error> {
            let mut size_buf = [0u8; 2];
            self.read_exact(&mut size_buf)?;
            let size = i16::from_le_bytes(size_buf);

            let mut opcode_buf = [0u8; 1];
            self.read_exact(&mut opcode_buf)?;
            let opcode = opcode_buf[0] as i8;

            if opcode != P::OPCODE {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Expected opcode {}, found {}", P::OPCODE, opcode),
                ));
            }

            let mut payload_buf = vec![0u8; (size - 3) as usize];
            self.read_exact(&mut payload_buf)?;

            P::deserialize(&payload_buf, ctx)
        }
    }

    impl<T: std::io::Read> ReadPacket for T {}
}
