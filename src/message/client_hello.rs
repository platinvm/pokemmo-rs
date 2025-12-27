use super::Message;

/// Initial greeting message sent by the client to the server.
///
/// `ClientHello` carries obfuscated integrity and timestamp values that are used
/// for secure challenge-response authentication. The values are obfuscated using
/// XOR operations with client-provided constants.
///
/// # Fields
///
/// - `obfuscated_integrity`: The integrity value XORed with the primary obfuscation constant.
/// - `obfuscated_timestamp`: The timestamp XORed with integrity and the secondary obfuscation constant.
#[derive(Message)]
pub struct ClientHello {
    obfuscated_integrity: i64,
    obfuscated_timestamp: i64,
}

impl ClientHello {
    /// Creates a new `ClientHello` message with the given parameters.\n    ///
    /// # Arguments
    ///
    /// - `integrity`: A unique value representing client integrity (e.g., derived from a pointer).
    /// - `timestamp`: The current system time.
    /// - `primary_obfuscation_value`: XOR constant to obfuscate the integrity value.
    /// - `secondary_obfuscation_value`: XOR constant to obfuscate the timestamp with integrity.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - System time cannot be accessed or calculated relative to UNIX_EPOCH.
    /// - The timestamp in milliseconds exceeds the range of `i64`.
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

    /// Recovers the original integrity value by de-obfuscating with the primary constant.
    ///
    /// # Arguments
    ///
    /// - `primary_obfuscation_value`: The same constant used in `new()`.
    pub fn integrity(&self, primary_obfuscation_value: i64) -> i64 {
        self.obfuscated_integrity ^ primary_obfuscation_value
    }

    /// Recovers the original timestamp by de-obfuscating with integrity and secondary constant.
    ///
    /// # Arguments
    ///
    /// - `primary_obfuscation_value`: The same constant used in `new()`.
    /// - `secondary_obfuscation_value`: The same constant used in `new()`.
    ///
    /// # Errors
    ///
    /// Returns an error if the recovered timestamp is negative and cannot be converted to `u64`.
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