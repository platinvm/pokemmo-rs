use p256::PublicKey;

#[derive(Debug, Clone)]
pub struct ClientReady {
    public_key: Vec<u8>,
}

impl ClientReady {
    pub fn new(public_key: PublicKey) -> Self {
        ClientReady {
            public_key: public_key.to_sec1_bytes().to_vec(),
        }
    }

    pub fn public_key(&self) -> Result<PublicKey, &'static str> {
        PublicKey::from_sec1_bytes(self.public_key.as_ref())
            .map_err(|_| "Failed to parse public key from bytes")
    }
}

impl TryFrom<crate::packet::Payload> for ClientReady {
    type Error = std::io::Error;

    fn try_from(payload: crate::packet::Payload) -> Result<Self, Self::Error> {
        if payload.opcode != 0x02 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid opcode for ClientReady message",
            ));
        }

        if payload.data.len() < 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Insufficient data for ClientReady message",
            ));
        }

        let pk_size = i16::from_le_bytes([payload.data[0], payload.data[1]]) as usize;

        if payload.data.len() < 2 + pk_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Insufficient data for public key in ClientReady message",
            ));
        }

        let public_key = payload.data[2..2 + pk_size].to_vec();

        Ok(ClientReady { public_key })
    }
}

impl TryInto<crate::packet::Payload> for ClientReady {
    type Error = std::io::Error;

    fn try_into(self) -> std::io::Result<crate::packet::Payload> {
        use std::io::Write;

        let mut data = Vec::new();
        let pk_size: i16 = self
            .public_key
            .len()
            .try_into()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
        data.write(&pk_size.to_le_bytes())?;
        data.write(&self.public_key)?;

        Ok(crate::packet::Payload { opcode: 0x02, data })
    }
}
