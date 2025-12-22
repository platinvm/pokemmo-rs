#[derive(Debug, Clone)]
pub struct ServerHello {
    public_key: Vec<u8>,
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

impl Into<crate::packet::Checksum> for Checksum {
    fn into(self) -> crate::packet::Checksum {
        match self {
            Checksum::None => crate::packet::Checksum::None,
            _ => crate::packet::Checksum::None,
        }
    }
}

impl TryFrom<crate::packet::Payload> for ServerHello {
    type Error = std::io::Error;

    fn try_from(payload: crate::packet::Payload) -> std::io::Result<Self> {
        use std::io::Read;

        if payload.opcode != 0x01 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid opcode for ServerHello",
            ));
        }

        let mut rdr = std::io::Cursor::new(payload.data);

        let mut size_buf = [0u8; 2];
        rdr.read_exact(&mut size_buf)?;
        let size = i16::from_le_bytes(size_buf) as usize;
        let mut public_key = vec![0u8; size];
        rdr.read_exact(&mut public_key)?;

        let sig_size = i16::from_le_bytes(size_buf) as usize;
        let mut signature = vec![0u8; sig_size];
        rdr.read_exact(&mut signature)?;

        let mut checksum_buf = [0u8; 1];
        rdr.read_exact(&mut checksum_buf)?;
        let checksum_size = checksum_buf[0] as i8;

        Ok(ServerHello {
            public_key,
            signature,
            checksum_size,
        })
    }
}

impl TryInto<crate::packet::Payload> for ServerHello {
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

        let sig_size: i16 = self
            .signature
            .len()
            .try_into()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

        data.write(&sig_size.to_le_bytes())?;
        data.write(&self.signature)?;

        data.write(&[self.checksum_size as u8])?;

        Ok(crate::packet::Payload { opcode: 0x01, data })
    }
}
