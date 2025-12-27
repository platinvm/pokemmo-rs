pub enum Login {
    ClientReady(crate::message::ClientReady),
    ServerHello(crate::message::ServerHello),
    ClientHello(crate::message::ClientHello),
}

impl super::Codec for Login {
    fn encode(&self) -> std::io::Result<Vec<u8>> {
        use crate::message::Message;

        Ok(match self {
            Login::ClientHello(msg) => {
                let mut msg_data = vec![0x00];
                msg_data.extend_from_slice(&msg.serialize()?);
                msg_data
            }
            Login::ServerHello(msg) => {
                let mut msg_data = vec![0x01];
                msg_data.extend_from_slice(&msg.serialize()?);
                msg_data
            }
            Login::ClientReady(msg) => {
                let mut msg_data = vec![0x02];
                msg_data.extend_from_slice(&msg.serialize()?);
                msg_data
            }
        })
    }

    fn decode(data: &[u8]) -> std::io::Result<Self> {
        use crate::message::Message;

        if data.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No opcode found in message",
            ));
        }

        match data[0] {
            0x00 => Ok(Login::ClientHello(
                crate::message::ClientHello::deserialize(&data[1..])?,
            )),
            0x01 => Ok(Login::ServerHello(
                crate::message::ServerHello::deserialize(&data[1..])?,
            )),
            0x02 => Ok(Login::ClientReady(
                crate::message::ClientReady::deserialize(&data[1..])?,
            )),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unknown message opcode",
            )),
        }
    }
}

impl Into<Login> for crate::message::ClientReady {
    fn into(self) -> Login {
        Login::ClientReady(self)
    }
}

impl Into<Login> for crate::message::ServerHello {
    fn into(self) -> Login {
        Login::ServerHello(self)
    }
}

impl Into<Login> for crate::message::ClientHello {
    fn into(self) -> Login {
        Login::ClientHello(self)
    }
}

impl TryFrom<Login> for crate::message::ClientReady {
    type Error = ();

    fn try_from(value: Login) -> Result<Self, Self::Error> {
        match value {
            Login::ClientReady(msg) => Ok(msg),
            _ => Err(()),
        }
    }
}

impl TryFrom<Login> for crate::message::ServerHello {
    type Error = ();

    fn try_from(value: Login) -> Result<Self, Self::Error> {
        match value {
            Login::ServerHello(msg) => Ok(msg),
            _ => Err(()),
        }
    }
}

impl TryFrom<Login> for crate::message::ClientHello {
    type Error = ();

    fn try_from(value: Login) -> Result<Self, Self::Error> {
        match value {
            Login::ClientHello(msg) => Ok(msg),
            _ => Err(()),
        }
    }
}
