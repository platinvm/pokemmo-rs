pub mod client_hello;
pub mod client_ready;
pub mod server_hello;
pub mod unknown;

use p256::ecdsa::Signature;
use p256::{PublicKey, SecretKey};

pub trait Payload: Sized {
    const OPCODE: i8;

    fn encode_payload(&self, data: impl std::io::Write, ctx: &Context) -> Result<(), std::io::Error>;
    fn decode_payload(data: impl std::io::Read, ctx: &Context) -> Result<Self, std::io::Error>;
}

pub struct Context {
    pub primary_obfuscation_value: i64,
    pub secondary_obfuscation_value: i64,

    pub checksum_size: Option<i8>,

    pub server_signature: Option<Signature>,
    pub server_public_key: Option<PublicKey>,
    pub server_secret_key: Option<SecretKey>,

    pub client_public_key: Option<PublicKey>,
    pub client_secret_key: Option<SecretKey>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            primary_obfuscation_value: 3214621489648854472,
            secondary_obfuscation_value: -4214651440992349575,

            checksum_size: None,

            server_signature: None,
            server_public_key: None,
            server_secret_key: None,

            client_public_key: None,
            client_secret_key: None,
        }
    }
}
