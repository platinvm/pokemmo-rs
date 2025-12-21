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

use super::Message;

impl From<ClientReady> for Message {
    fn from(msg: ClientReady) -> Self {
        Message::ClientReady {
            public_key: msg.public_key,
        }
    }
}

impl TryFrom<Message> for ClientReady {
    type Error = &'static str;
    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        if let Message::ClientReady { public_key } = msg {
            Ok(ClientReady { public_key })
        } else {
            Err("Not a ClientReady message")
        }
    }
}
