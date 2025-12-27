use super::codec;

/// The login protocol codec defining the message types exchanged during authentication.
///
/// The login sequence is:
/// 1. Client sends `ClientHello` (opcode 0x00)
/// 2. Server responds with `ServerHello` (opcode 0x01)
/// 3. Client acknowledges with `ClientReady` (opcode 0x02)
///
/// Any unrecognized opcode is captured as `Unknown`, carrying its own opcode as `i8`
/// and the raw payload data for extensibility and debugging.
#[codec]
pub enum Login {
    /// Client's initial greeting message.
    ClientHello(crate::message::ClientHello) = 0x00u8,
    /// Server's response with cryptographic material.
    ServerHello(crate::message::ServerHello) = 0x01u8,
    /// Client's acknowledgement with its public key.
    ClientReady(crate::message::ClientReady) = 0x02u8,
    /// Unrecognized message variant for future extensibility.
    ///
    /// - `opcode`: The raw opcode byte (signed, allowing negative values).
    /// - `data`: The unparsed payload.
    Unknown { opcode: i8, data: Vec<u8> },
}
