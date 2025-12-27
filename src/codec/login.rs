use super::codec;

#[codec]
pub enum Login {
    ClientHello(crate::message::ClientHello) = 0x00,
    ServerHello(crate::message::ServerHello) = 0x01,
    ClientReady(crate::message::ClientReady) = 0x02,
}
