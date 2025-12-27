use super::Message;

#[derive(Message)]
pub struct ClientReady {
    #[prefixed(i16)]
    public_key: Vec<u8>,
}

impl ClientReady {
    pub fn new(public_key: p256::PublicKey) -> Self {
        ClientReady {
            public_key: public_key.to_sec1_bytes().to_vec(),
        }
    }

    pub fn public_key(&self) -> Result<p256::PublicKey, &'static str> {
        p256::PublicKey::from_sec1_bytes(self.public_key.as_ref())
            .map_err(|_| "Failed to parse public key from bytes")
    }
}
