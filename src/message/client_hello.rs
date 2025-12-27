use super::Message;

/// ClientHello (opcode 0x00) initiates the CTLS handshake as per pokemmo-spec.
///
/// The client sends two 8-byte values:
/// - Obfuscated Random Key (spec): random value XOR'd with `key1`
/// - Obfuscated Timestamp (spec): timestamp XOR'd with `key2` and the random key
///
/// In this implementation, `obfuscated_integrity` represents the random key obfuscated
/// with `primary_obfuscation_value` (spec: `key1`), and `obfuscated_timestamp` is
/// obfuscated with both the integrity value and `secondary_obfuscation_value` (spec: `key2`).
#[derive(Message)]
pub struct ClientHello {
    obfuscated_integrity: i64,
    obfuscated_timestamp: i64,
}

impl ClientHello {
    /// Creates a new `ClientHello` following the spec obfuscation scheme.
    ///
    /// Arguments:
    /// - `integrity` (spec: random key): any 64-bit value, XOR'd with `primary_obfuscation_value` (key1).
    /// - `timestamp`: current `SystemTime`, encoded as milliseconds since UNIX_EPOCH.
    /// - `primary_obfuscation_value` (spec: key1): 64-bit obfuscation constant.
    /// - `secondary_obfuscation_value` (spec: key2): 64-bit obfuscation constant.
    ///
        /// ## Errors:
    /// - Fails if `timestamp` cannot be represented as `i64` milliseconds.
    pub fn new(
        integrity: i64,
        timestamp: std::time::SystemTime,
        primary_obfuscation_value: i64,
        secondary_obfuscation_value: i64,
    ) -> Result<Self, &'static str> {
        let timestamp_millis: i64 = timestamp
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map_err(|_| "Time went backwards")?
            .as_millis()
            .try_into()
            .map_err(|_| "Timestamp too large to fit in i64")?;

        Ok(ClientHello {
            obfuscated_integrity: integrity ^ primary_obfuscation_value,
            obfuscated_timestamp: timestamp_millis ^ integrity ^ secondary_obfuscation_value,
        })
    }

    /// Recovers the original random key (integrity) by de-obfuscating with key1.
    ///
    /// Arguments:
    /// - `primary_obfuscation_value` (spec: key1)
    pub fn integrity(&self, primary_obfuscation_value: i64) -> i64 {
        self.obfuscated_integrity ^ primary_obfuscation_value
    }

    /// Recovers the original timestamp by de-obfuscating with integrity and key2.
    ///
    /// Arguments:
    /// - `primary_obfuscation_value` (spec: key1)
    /// - `secondary_obfuscation_value` (spec: key2)
    ///
        /// ## Errors:
    /// - Fails if recovered milliseconds are out of range for `u64`.
    pub fn timestamp(
        &self,
        primary_obfuscation_value: i64,
        secondary_obfuscation_value: i64,
    ) -> Result<std::time::SystemTime, &'static str> {
        let timestamp_millis = self.obfuscated_timestamp
            ^ self.integrity(primary_obfuscation_value)
            ^ secondary_obfuscation_value;

        let millis: u64 = timestamp_millis
            .try_into()
            .map_err(|_| "Timestamp out of range")?;

        Ok(std::time::UNIX_EPOCH + std::time::Duration::from_millis(millis))
    }
}