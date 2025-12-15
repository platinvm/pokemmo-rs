use p256::ecdsa::Signature;
use p256::elliptic_curve::rand_core::OsRng;
use p256::PublicKey;
use p256::SecretKey;

pub struct ServerHello {
    public_key: PublicKey,
    signature: Signature,
    checksum_size: i8,
}

impl Default for ServerHello {
    fn default() -> Self {
        // Generate a temporary keypair for default
        let secret_key = SecretKey::random(&mut OsRng);
        let public_key = secret_key.public_key();
        // Create a dummy signature (this default is mainly for trait compliance)
        use p256::ecdsa::{signature::Signer, SigningKey};
        let signing_key = SigningKey::from(&secret_key);
        let signature: Signature = signing_key.sign(&[0u8; 32]);
        Self {
            public_key,
            signature,
            checksum_size: 0,
        }
    }
}

impl ServerHello {
    pub fn new(public_key: PublicKey, signature: Signature, checksum_size: i8) -> Self {
        Self {
            public_key,
            signature,
            checksum_size: checksum_size,
        }
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn checksum_size(&self) -> i8 {
        self.checksum_size
    }
}

crate::payload! {
    ServerHello {
        const OPCODE: i8 = 0x01;
        type Context = ();

        fn serialize(&self, _ctx: &Self::Context) -> std::io::Result<Vec<u8>> {
            use p256::elliptic_curve::sec1::ToEncodedPoint;

            let mut data = Vec::new();

            let uncompressed_point = self.public_key.to_encoded_point(false);
            let uncompressed_point_bytes = uncompressed_point.as_bytes();

            let uncompressed_point_size = uncompressed_point_bytes.len() as i16;
            data.extend_from_slice(&uncompressed_point_size.to_le_bytes());
            data.extend_from_slice(uncompressed_point_bytes);

            let signature_bytes = self.signature.to_der();
            let signature_size = signature_bytes.as_bytes().len() as i16;
            data.extend_from_slice(&signature_size.to_le_bytes());
            data.extend_from_slice(signature_bytes.as_bytes());

            data.extend_from_slice(&self.checksum_size.to_le_bytes());

            Ok(data)
        }

        fn deserialize(data: &[u8], _ctx: &Self::Context) -> std::io::Result<Self> {
            use p256::elliptic_curve::sec1::FromEncodedPoint;
            use p256::EncodedPoint;

            let mut cursor = 0;

            if data.len() < 2 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Not enough data for public key size",
                ));
            }
            let uncompressed_point_size = i16::from_le_bytes(data[cursor..cursor + 2].try_into().unwrap()) as usize;
            cursor += 2;

            if data.len() < cursor + uncompressed_point_size {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Not enough data for public key",
                ));
            }
            let uncompressed_point_bytes = &data[cursor..cursor + uncompressed_point_size];
            cursor += uncompressed_point_size;

            let encoded_point = EncodedPoint::from_bytes(uncompressed_point_bytes)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            let public_key = PublicKey::from_encoded_point(&encoded_point)
                .into_option()
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid public key")
                })?;

            if data.len() < cursor + 2 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Not enough data for signature size",
                ));
            }
            let signature_size = i16::from_le_bytes(data[cursor..cursor + 2].try_into().unwrap()) as usize;
            cursor += 2;

            if data.len() < cursor + signature_size {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Not enough data for signature",
                ));
            }
            let signature_bytes = &data[cursor..cursor + signature_size];
            cursor += signature_size;

            let signature = Signature::from_der(signature_bytes)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            // TODO: Verify the signature against a trusted public key to ensure server authenticity
            // This would prevent man-in-the-middle attacks. We need to:
            // 1. Determine what data the signature is signing (likely the public_key)
            // 2. Obtain a trusted/root public key for verification
            // 3. Use signature.verify() or similar to validate

            if data.len() < cursor + 1 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Not enough data for hash size",
                ));
            }
            let hash_size = i8::from_le_bytes(data[cursor..cursor + 1].try_into().unwrap());

            Ok(Self {
                public_key,
                signature,
                checksum_size: hash_size,
            })
        }
    }
}
