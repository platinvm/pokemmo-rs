use crate::message::Message;

#[derive(Message)]
pub struct ServerHello {
    #[prefixed(i16)]
    public_key: Vec<u8>,
    #[prefixed(i16)]
    signature: Vec<u8>,
    checksum_size: i8,
}

impl ServerHello {
    pub fn new(
        public_key: p256::PublicKey,
        signature: p256::ecdsa::Signature,
        checksum: Checksum,
    ) -> Self {
        ServerHello {
            public_key: public_key.to_sec1_bytes().to_vec(),
            signature: signature.to_der().as_bytes().to_vec(),
            checksum_size: checksum.into(),
        }
    }

    pub fn public_key(&self) -> Result<p256::PublicKey, &'static str> {
        p256::PublicKey::from_sec1_bytes(self.public_key.as_ref())
            .map_err(|_| "Failed to parse public key from bytes")
    }

    pub fn signature(&self) -> Result<p256::ecdsa::Signature, &'static str> {
        p256::ecdsa::Signature::from_der(self.signature.as_ref())
            .map_err(|_| "Failed to parse signature from bytes")
    }

    pub fn checksum(&self) -> Result<Checksum, &'static str> {
        Checksum::try_from(self.checksum_size)
    }
}

#[derive(Debug, Clone)]
pub enum Checksum {
    None,
    Crc16,
    HmacSha256(i8),
}

impl TryFrom<i8> for Checksum {
    type Error = &'static str;
    fn try_from(size: i8) -> Result<Self, Self::Error> {
        Ok(match size {
            0 => Checksum::None,
            1 => Checksum::Crc16,
            4..=32 => Checksum::HmacSha256(size),
            _ => return Err("Invalid checksum size"),
        })
    }
}

impl Into<i8> for Checksum {
    fn into(self) -> i8 {
        match self {
            Checksum::None => 0,
            Checksum::Crc16 => 1,
            Checksum::HmacSha256(size) => size,
        }
    }
}
