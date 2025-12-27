use super::codec;

/// Login protocol codec (handshake opcodes per pokemmo-spec).
///
/// Sequence:
/// - `ClientHello` (0x00): initiates handshake with obfuscated values
/// - `ServerHello` (0x01): returns server public key, signature, checksum size
/// - `ClientReady` (0x02): sends client public key to complete handshake
///
/// Unrecognized opcodes fall back to `Unknown`, which carries its own opcode (`i8`, LE)
/// and the raw payload for debugging/extensibility.
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
    /// - `opcode` (i8 LE): raw opcode byte (signed)
    /// - `data`: unparsed payload bytes
    Unknown { opcode: i8, data: Vec<u8> },
}
