use crate::{Context, Payload};
use p256::SecretKey;

pub struct ClientReady {
    secret_key: SecretKey,
}

impl ClientReady {
    pub fn new(secret_key: SecretKey) -> Self {
        Self { secret_key: secret_key }
    }

    pub fn secret_key(&self) -> &SecretKey {
        &self.secret_key
    }
}

impl Default for ClientReady {
    fn default() -> Self {
        Self {
            secret_key: SecretKey::random(&mut rand::thread_rng()),
        }
    }
}

impl Payload for ClientReady {
    const OPCODE: i8 = 0x02;

    fn encode(&self, mut data: impl std::io::Write, _ctx: &Context) -> Result<(), std::io::Error> {
        use p256::elliptic_curve::sec1::ToEncodedPoint;

        let public_key = self.secret_key.public_key();
        let uncompressed_point = public_key.to_encoded_point(false);
        let uncompressed_point_bytes = uncompressed_point.as_bytes();

        let uncompressed_point_size = uncompressed_point_bytes.len() as i16;
        data.write_all(&uncompressed_point_size.to_le_bytes())?;
        data.write_all(uncompressed_point_bytes)?;

        Ok(())
    }

    fn decode(data: impl std::io::Read, _ctx: &Context) -> Result<Self, std::io::Error> {
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

        let _public_key = PublicKey::from_encoded_point(&encoded_point)
            .into_option()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid public key")
            })?;

        // Note: We can't reconstruct the private key from the public key
        // This is only used for decoding received data, which doesn't make sense for ClientReady
        // as it's a client->server message. Keeping this for API consistency.
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Cannot decode ClientReady - private key cannot be reconstructed from public key",
        ))
    }
}
