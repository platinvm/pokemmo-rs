use crate::payload;
use p256::PublicKey;

pub struct ClientReady {
    public_key: PublicKey,
}

impl ClientReady {
    pub fn new(public_key: PublicKey) -> Self {
        Self { public_key }
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

impl payload::Payload for ClientReady {
    const OPCODE: i8 = 0x02;

    fn encode_payload(&self, mut data: impl std::io::Write, _ctx: &payload::Context) -> Result<(), std::io::Error> {
        use p256::elliptic_curve::sec1::ToEncodedPoint;

        let uncompressed_point = self.public_key.to_encoded_point(false);
        let uncompressed_point_bytes = uncompressed_point.as_bytes();

        let uncompressed_point_size = uncompressed_point_bytes.len() as i16;
        data.write_all(&uncompressed_point_size.to_le_bytes())?;
        data.write_all(uncompressed_point_bytes)?;

        Ok(())
    }

    fn decode_payload(data: impl std::io::Read, _ctx: &payload::Context) -> Result<Self, std::io::Error> {
        use p256::elliptic_curve::sec1::FromEncodedPoint;
        use p256::{EncodedPoint, PublicKey};

        let mut uncompressed_point_size_buf = [0u8; 2];

        let mut reader = data;
        reader.read_exact(&mut uncompressed_point_size_buf)?;

        let uncompressed_point_size = i16::from_le_bytes(uncompressed_point_size_buf);
        let mut uncompressed_point_bytes = vec![0u8; uncompressed_point_size as usize];
        reader.read_exact(&mut uncompressed_point_bytes)?;

        let encoded_point = EncodedPoint::from_bytes(&uncompressed_point_bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let public_key = PublicKey::from_encoded_point(&encoded_point)
            .into_option()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid public key")
            })?;

        Ok(Self { public_key })
    }
}
