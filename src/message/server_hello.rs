use p256::{ecdsa::Signature, PublicKey};

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

#[derive(Debug, Clone)]
pub struct ServerHello {
    public_key: Vec<u8>,
    signature: Vec<u8>,
    checksum_size: i8,
}

impl ServerHello {
    pub fn new(public_key: PublicKey, signature: Signature, checksum: Checksum) -> Self {
        ServerHello {
            public_key: public_key.to_sec1_bytes().to_vec(),
            signature: signature.to_der().as_bytes().to_vec(),
            checksum_size: checksum.into(),
        }
    }

    pub fn public_key(&self) -> Result<PublicKey, &'static str> {
        PublicKey::from_sec1_bytes(self.public_key.as_ref())
            .map_err(|_| "Failed to parse public key from bytes")
    }

    pub fn signature(&self) -> Result<Signature, &'static str> {
        Signature::from_der(self.signature.as_ref())
            .map_err(|_| "Failed to parse signature from bytes")
    }

    pub fn checksum(&self) -> Result<Checksum, &'static str> {
        Checksum::try_from(self.checksum_size)
    }
}

use super::Message;

impl From<ServerHello> for Message {
    fn from(msg: ServerHello) -> Self {
        Message::ServerHello {
            public_key: msg.public_key,
            signature: msg.signature,
            checksum_size: msg.checksum_size,
        }
    }
}

impl TryFrom<Message> for ServerHello {
    type Error = &'static str;
    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        if let Message::ServerHello {
            public_key,
            signature,
            checksum_size,
        } = msg
        {
            Ok(ServerHello {
                public_key,
                signature,
                checksum_size,
            })
        } else {
            Err("Not a ServerHello message")
        }
    }
}
