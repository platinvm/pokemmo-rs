use crate::{Context, Packet, Payload};

/// Extension trait for writing packets to streams.
pub trait WriteFrameExt: std::io::Write {
    fn write_packet<P: Payload, B: Packet<P>>(
        &mut self,
        packet: &B,
        ctx: &Context,
    ) -> Result<(), std::io::Error> {
        let mut packet_buf = Vec::new();
        packet.encode(&mut packet_buf, ctx)?;

        let size = packet_buf.len() as i16 + 2;
        self.write_all(&size.to_le_bytes())?;
        self.write_all(&packet_buf)?;

        self.flush()?;

        Ok(())
    }
}

impl<T: std::io::Write> WriteFrameExt for T {}

/// Extension trait for reading packets from streams.
pub trait ReadFrameExt: std::io::Read {
    fn read_packet<P: Payload, B: Packet<P>>(
        &mut self,
        ctx: &Context,
    ) -> Result<B, std::io::Error> {
        let mut size_buf = [0u8; 2];
        self.read_exact(&mut size_buf)?;
        let size = i16::from_le_bytes(size_buf);

        let mut packet_buf = vec![0u8; (size - 2) as usize];
        self.read_exact(&mut packet_buf)?;

        B::decode(&packet_buf[..], ctx)
    }
}

impl<T: std::io::Read> ReadFrameExt for T {}
