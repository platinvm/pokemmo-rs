use crate::payload;

pub trait Packet<P: payload::Payload>: Sized {
    fn encode_packet(
        &self,
        data: impl std::io::Write,
        ctx: &payload::Context,
    ) -> Result<(), std::io::Error>;
    fn decode_packet(
        data: impl std::io::Read,
        ctx: &payload::Context,
    ) -> Result<Self, std::io::Error>;
}

impl<P: payload::Payload> Packet<P> for P {
    fn encode_packet(
        &self,
        mut data: impl std::io::Write,
        ctx: &payload::Context,
    ) -> Result<(), std::io::Error> {
        data.write_all(&[P::OPCODE as u8])?;
        payload::Payload::encode_payload(self, data, ctx)
    }

    fn decode_packet(
        mut data: impl std::io::Read,
        ctx: &payload::Context,
    ) -> Result<Self, std::io::Error> {
        let mut opcode_buf = [0u8; 1];
        data.read_exact(&mut opcode_buf)?;
        let opcode = opcode_buf[0] as i8;
        if opcode != P::OPCODE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Expected opcode {}, found {}", P::OPCODE, opcode),
            ));
        }

        P::decode_payload(data, ctx)
    }
}

pub mod ext {
    use crate::packet;
    use crate::payload;

    pub trait WritePacket: std::io::Write {
        fn write_packet<P: payload::Payload, B: packet::Packet<P>>(
            &mut self,
            packet: &B,
            ctx: &payload::Context,
        ) -> Result<(), std::io::Error> {
            let mut packet_buf = Vec::new();
            packet.encode_packet(&mut packet_buf, ctx)?;

            let size = packet_buf.len() as i16 + 2;
            self.write_all(&size.to_le_bytes())?;
            self.write_all(&packet_buf)?;

            self.flush()?;

            Ok(())
        }
    }

    impl<T: std::io::Write> WritePacket for T {}

    pub trait ReadPacket: std::io::Read {
        fn read_packet<P: payload::Payload, B: packet::Packet<P>>(
            &mut self,
            ctx: &payload::Context,
        ) -> Result<B, std::io::Error> {
            let mut size_buf = [0u8; 2];
            self.read_exact(&mut size_buf)?;
            let size = i16::from_le_bytes(size_buf);

            let mut packet_buf = vec![0u8; (size - 2) as usize];
            self.read_exact(&mut packet_buf)?;

            B::decode_packet(&packet_buf[..], ctx)
        }
    }

    impl<T: std::io::Read> ReadPacket for T {}
}
