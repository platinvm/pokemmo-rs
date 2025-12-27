use super::Message;

/// Client's acknowledgement after receiving `ServerHello`, providing its public key.
///
/// `ClientReady` contains the client's ECDSA public key in SEC1 format, allowing
/// the server to establish a shared secret for encrypted communication.
///
/// # Fields
///
/// - `public_key`: The client's P-256 public key in SEC1 format, prefixed with a 2-byte length.
#[derive(Message)]
pub struct ClientReady {
    #[prefixed(i16)]
    public_key: Vec<u8>,
}

impl ClientReady {
    /// Creates a new `ClientReady` message with the client's public key.
    ///
    /// # Arguments
    ///
    /// - `public_key`: The client's P-256 public key.
    pub fn new(public_key: p256::PublicKey) -> Self {
        ClientReady {
            public_key: public_key.to_sec1_bytes().to_vec(),
        }
    }

    /// Parses the client's public key from its SEC1-encoded bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes are not a valid P-256 SEC1 public key.
    pub fn public_key(&self) -> Result<p256::PublicKey, &'static str> {
        p256::PublicKey::from_sec1_bytes(self.public_key.as_ref())
            .map_err(|_| "Failed to parse public key from bytes")
    }
}
