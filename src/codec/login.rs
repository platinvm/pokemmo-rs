use super::codec;

#[codec]
pub enum Login {
    ClientHello(crate::message::ClientHello) = 0x00u8,
    ServerHello(crate::message::ServerHello) = 0x01u8,
    ClientReady(crate::message::ClientReady) = 0x02u8,
    Unknown { opcode: i8, data: Vec<u8> },
}
