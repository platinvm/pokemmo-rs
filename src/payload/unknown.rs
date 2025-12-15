use crate::payload;

pub type Unknown = Vec<u8>;

impl payload::Payload for Unknown {
    const OPCODE: i8 = i8::MIN;

    fn encode_payload(
        &self,
        mut data: impl std::io::Write,
        _ctx: &payload::Context,
    ) -> Result<(), std::io::Error> {
        data.write_all(self)?;
        Ok(())
    }

    fn decode_payload(data: impl std::io::Read, _ctx: &payload::Context) -> Result<Self, std::io::Error> {
        let mut buf = Vec::new();
        let mut reader = data;
        reader.read_to_end(&mut buf)?;
        Ok(buf)
    }
}
