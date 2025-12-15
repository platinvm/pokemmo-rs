use p256::elliptic_curve::rand_core::OsRng;
use p256::PublicKey;
use p256::SecretKey;

pub struct ClientReady {
    public_key: PublicKey,
}

impl Default for ClientReady {
    fn default() -> Self {
        // Generate a temporary keypair for default
        let secret_key = SecretKey::random(&mut OsRng);
        let public_key = secret_key.public_key();
        Self { public_key }
    }
}

impl ClientReady {
    pub fn new(public_key: PublicKey) -> Self {
        Self { public_key }
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

crate::payload! {
    ClientReady {
        const OPCODE: i8 = 0x02;
        type Context = ();
        type Error = std::io::Error;

        fn serialize(&self, _ctx: &Self::Context) -> Result<Vec<u8>, Self::Error> {
            use p256::elliptic_curve::sec1::ToEncodedPoint;

            let mut data = Vec::new();

            let uncompressed_point = self.public_key.to_encoded_point(false);
            let uncompressed_point_bytes = uncompressed_point.as_bytes();

            let uncompressed_point_size = uncompressed_point_bytes.len() as i16;
            data.extend_from_slice(&uncompressed_point_size.to_le_bytes());
            data.extend_from_slice(uncompressed_point_bytes);

            Ok(data)
        }

        fn deserialize(data: &[u8], _ctx: &Self::Context) -> Result<Self, Self::Error> {
            use p256::elliptic_curve::sec1::FromEncodedPoint;
            use p256::{EncodedPoint, PublicKey};

            if data.len() < 2 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Not enough data for public key size",
                ));
            }

            let uncompressed_point_size = i16::from_le_bytes(data[0..2].try_into().unwrap()) as usize;

            if data.len() < 2 + uncompressed_point_size {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Not enough data for public key",
                ));
            }

            let uncompressed_point_bytes = &data[2..2 + uncompressed_point_size];

            let encoded_point = EncodedPoint::from_bytes(uncompressed_point_bytes)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            let public_key = PublicKey::from_encoded_point(&encoded_point)
                .into_option()
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid public key")
                })?;

            Ok(Self { public_key })
        }
    }
}
