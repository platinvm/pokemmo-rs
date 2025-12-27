use super::Message;

/// Server's response to `ClientHello`, containing cryptographic material and checksum configuration.
///
/// `ServerHello` provides the server's public key and a signature for establishing
/// encrypted communication. It also specifies the checksum algorithm to use for message integrity.
///
/// # Fields
///
/// - `public_key`: The server's ECDSA public key in SEC1 format, prefixed with a 2-byte length.
/// - `signature`: The server's ECDSA signature over its public key, in DER format, prefixed with a 2-byte length.
/// - `checksum_size`: Configuration for message integrity checks (None, CRC16, or HMAC-SHA256).
#[derive(Message)]
pub struct ServerHello {
    #[prefixed(i16)]
    public_key: Vec<u8>,
    #[prefixed(i16)]
    signature: Vec<u8>,
    checksum_size: i8,
}

impl ServerHello {
    /// Creates a new `ServerHello` message from cryptographic components.
    ///
    /// # Arguments
    ///
    /// - `public_key`: The server's P-256 public key.
    /// - `signature`: The server's ECDSA signature over the public key.
    /// - `checksum`: The checksum configuration for the session.
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

    /// Parses the server's public key from its SEC1-encoded bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes are not a valid P-256 SEC1 public key.
    pub fn public_key(&self) -> Result<p256::PublicKey, &'static str> {
        p256::PublicKey::from_sec1_bytes(self.public_key.as_ref())
            .map_err(|_| "Failed to parse public key from bytes")
    }

    /// Parses the server's signature from its DER-encoded bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes are not a valid DER-encoded ECDSA signature.
    pub fn signature(&self) -> Result<p256::ecdsa::Signature, &'static str> {
        p256::ecdsa::Signature::from_der(self.signature.as_ref())
            .map_err(|_| "Failed to parse signature from bytes")
    }

    /// Parses the checksum configuration from the encoded byte.
    ///
    /// # Errors
    ///
    /// Returns an error if the checksum size byte does not correspond to a valid configuration.
    pub fn checksum(&self) -> Result<Checksum, &'static str> {
        Checksum::try_from(self.checksum_size)
    }
}

#[derive(Debug, Clone)]
/// Specifies the message integrity check algorithm for the session.
///
/// - `None`: No integrity checking (size byte = 0).
/// - `Crc16`: CRC-16 checksum (size byte = 1).
/// - `HmacSha256(size)`: HMAC-SHA256 with configurable size, where `size` is 4-32 (size byte = 4..=32).
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
