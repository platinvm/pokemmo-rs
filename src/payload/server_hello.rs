use crate::payload;
use p256::ecdsa::Signature;
use p256::PublicKey;

pub struct ServerHello {
    public_key: PublicKey,
    signature: Signature,
    checksum_size: i8,
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

impl payload::Payload for ServerHello {
    const OPCODE: i8 = 0x01;

    fn encode_payload(
        &self,
        mut data: impl std::io::Write,
        _ctx: &payload::Context,
    ) -> Result<(), std::io::Error> {
        use p256::elliptic_curve::sec1::ToEncodedPoint;

        let uncompressed_point = self.public_key.to_encoded_point(false);
        let uncompressed_point_bytes = uncompressed_point.as_bytes();

        let uncompressed_point_size = uncompressed_point_bytes.len() as i16;
        data.write_all(&uncompressed_point_size.to_le_bytes())?;
        data.write_all(uncompressed_point_bytes)?;

        let signature_bytes = self.signature.to_der();
        let signature_size = signature_bytes.as_bytes().len() as i16;
        data.write_all(&signature_size.to_le_bytes())?;
        data.write_all(signature_bytes.as_bytes())?;

        data.write_all(&self.checksum_size.to_le_bytes())?;

        Ok(())
    }

    fn decode_payload(data: impl std::io::Read, _ctx: &payload::Context) -> Result<Self, std::io::Error> {
        use p256::elliptic_curve::sec1::FromEncodedPoint;
        use p256::EncodedPoint;

        let mut uncompressed_point_size_buf = [0u8; 2];
        let mut signature_size_buf = [0u8; 2];
        let mut hash_size_buf = [0u8; 1];

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

        reader.read_exact(&mut signature_size_buf)?;
        let signature_size = i16::from_le_bytes(signature_size_buf);
        let mut signature_bytes = vec![0u8; signature_size as usize];
        reader.read_exact(&mut signature_bytes)?;

        let signature = Signature::from_der(&signature_bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // TODO: Verify the signature against a trusted public key to ensure server authenticity
        // This would prevent man-in-the-middle attacks. We need to:
        // 1. Determine what data the signature is signing (likely the public_key)
        // 2. Obtain a trusted/root public key for verification
        // 3. Use signature.verify() or similar to validate

        reader.read_exact(&mut hash_size_buf)?;
        let hash_size = i8::from_le_bytes(hash_size_buf);

        Ok(Self {
            public_key,
            signature,
            checksum_size: hash_size,
        })
    }
}
