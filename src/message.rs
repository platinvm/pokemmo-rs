pub mod client_hello;
pub mod client_ready;
pub mod server_hello;

pub use client_hello::ClientHello;
pub use client_ready::ClientReady;
pub use server_hello::ServerHello;

pub enum Message {
    ClientHello(client_hello::ClientHello),
    ClientReady(client_ready::ClientReady),
    ServerHello(server_hello::ServerHello),
    Unknown(crate::packet::Payload),
}

impl TryFrom<crate::packet::Payload> for Message {
    type Error = std::io::Error;

    fn try_from(payload: crate::packet::Payload) -> std::io::Result<Self> {
        match payload.opcode {
            0x00 => Ok(Message::ClientHello(client_hello::ClientHello::try_from(
                payload,
            )?)),
            0x01 => Ok(Message::ServerHello(server_hello::ServerHello::try_from(
                payload,
            )?)),
            0x02 => Ok(Message::ClientReady(client_ready::ClientReady::try_from(
                payload,
            )?)),
            _ => Ok(Message::Unknown(payload)),
        }
    }
}

impl TryInto<crate::packet::Payload> for Message {
    type Error = std::io::Error;

    fn try_into(self) -> std::io::Result<crate::packet::Payload> {
        match self {
            Message::ClientHello(msg) => msg.try_into(),
            Message::ServerHello(msg) => msg.try_into(),
            Message::ClientReady(msg) => msg.try_into(),
            Message::Unknown(payload) => Ok(payload),
        }
    }
}

pub trait ReadMessage: crate::packet::ReadPacket {
    fn read_message<T>(
        &mut self,
        encryption: crate::packet::Encryption,
        checksum: crate::packet::Checksum,
    ) -> std::io::Result<T>
    where
        T: TryFrom<crate::packet::Payload, Error = std::io::Error>,
    {
        let packet = self.read_packet()?;
        let payload = packet.payload(encryption, checksum);
        T::try_from(payload)
    }
}

impl<T: crate::packet::ReadPacket> ReadMessage for T {}

pub trait WriteMessage: crate::packet::WritePacket {
    fn write_message<T: TryInto<crate::packet::Payload, Error = std::io::Error>>(
        &mut self,
        message: T,
        encryption: crate::packet::Encryption,
        checksum: crate::packet::Checksum,
    ) -> std::io::Result<()> {
        let payload = message.try_into()?;
        let packet = crate::packet::Packet::new(payload, encryption, checksum)?;
        self.write_packet(&packet)
    }
}

impl<T: crate::packet::WritePacket> WriteMessage for T {}
