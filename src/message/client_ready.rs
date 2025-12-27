pub struct ClientReady {
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

impl super::Message for ClientReady {
    fn deserialize(data: &[u8]) -> std::io::Result<Self> {
        if data.len() < 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Insufficient data for ClientReady message",
            ));
        }

        let pk_size = i16::from_le_bytes([data[0], data[1]]) as usize;

        if data.len() < 2 + pk_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Insufficient data for public key in ClientReady message",
            ));
        }

        let public_key = data[2..2 + pk_size].to_vec();

        Ok(ClientReady { public_key })
    }

    fn serialize(&self) -> std::io::Result<Vec<u8>> {
        use std::io::Write;

        let mut data = Vec::new();
        let pk_size: i16 = self
            .public_key
            .len()
            .try_into()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
        data.write_all(&pk_size.to_le_bytes())?;
        data.write_all(&self.public_key)?;

        Ok(data)
    }
}
