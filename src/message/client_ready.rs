use super::Message;

/// ClientReady (opcode 0x02) completes the handshake.
///
/// As per pokemmo-spec, the client sends its ECDSA public key (SEC1 uncompressed),
/// length-prefixed with a 2-byte little-endian field.
///
/// Fields:
/// - `public_key`: Client P-256 public key (SEC1), prefixed by `i16` length.
#[derive(Message)]
pub struct ClientReady {
    #[prefixed(i16)]
    public_key: Vec<u8>,
}

impl ClientReady {
    /// Creates a new `ClientReady` message with the client's public key.
    ///
    /// Arguments:
    /// - `public_key`: Client P-256 public key (SEC1 uncompressed).
    pub fn new(public_key: p256::PublicKey) -> Self {
        ClientReady {
            public_key: public_key.to_sec1_bytes().to_vec(),
        }
    }

    /// Parses the client's public key from its SEC1-encoded bytes.
    ///
    /// Errors:
    /// - Fails if bytes are not a valid P-256 SEC1 public key.
    pub fn public_key(&self) -> Result<p256::PublicKey, &'static str> {
        p256::PublicKey::from_sec1_bytes(self.public_key.as_ref())
            .map_err(|_| "Failed to parse public key from bytes")
    }
}
