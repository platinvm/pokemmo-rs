use super::Message;

/// ServerHello (opcode 0x01) responds with server authentication data.
///
/// As per pokemmo-spec, the server provides:
/// - Server Public Key (length-prefixed, SEC1 uncompressed)
/// - ECDSA Signature (length-prefixed)
/// - Checksum Size (1 byte) — negotiated integrity protection size
///
/// Notes:
/// - This implementation encodes the signature in DER; the spec allows variable-length signatures.
/// - Checksum size mapping per spec: NoOp=0, CRC16=2, HMAC-SHA256=4..=32 (default 16).
#[derive(Message)]
pub struct ServerHello {
    #[prefixed(i16)]
    public_key: Vec<u8>,
    #[prefixed(i16)]
    signature: Vec<u8>,
    checksum_size: i8,
}

impl ServerHello {
    /// Creates a new `ServerHello` with public key, signature, and checksum parameters.
    ///
    /// Arguments:
    /// - `public_key`: Server's P-256 public key (SEC1 uncompressed).
    /// - `signature`: ECDSA signature over the public key (DER-encoded here).
    /// - `checksum`: Negotiated checksum (NoOp, CRC16, HMAC-SHA256(4..=32)).
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
    /// Errors:
    /// - Fails if bytes are not a valid P-256 SEC1 public key.
    pub fn public_key(&self) -> Result<p256::PublicKey, &'static str> {
        p256::PublicKey::from_sec1_bytes(self.public_key.as_ref())
            .map_err(|_| "Failed to parse public key from bytes")
    }

    /// Parses the server's signature from its DER-encoded bytes.
    ///
    /// Errors:
    /// - Fails if bytes are not a valid DER-encoded ECDSA signature.
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
/// Message integrity configuration negotiated during handshake.
///
/// Spec mapping:
/// - `None`: No checksum (`checksum_size = 0`)
/// - `Crc16`: CRC16 (`checksum_size = 2`)
/// - `HmacSha256(size)`: HMAC-SHA256 with `size` in 4..=32 (`checksum_size = size`)
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
