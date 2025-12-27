pub struct ClientHello {
    obfuscated_integrity: i64,
    obfuscated_timestamp: i64,
}

impl ClientHello {
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

    pub fn integrity(&self, primary_obfuscation_value: i64) -> i64 {
        self.obfuscated_integrity ^ primary_obfuscation_value
    }

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

impl super::Message for ClientHello {
    fn deserialize(data: &[u8]) -> std::io::Result<Self> {
        if data.len() != 16 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid data length for ClientHello message",
            ));
        }

        let obfuscated_integrity = i64::from_le_bytes(data[0..8].try_into().unwrap());
        let obfuscated_timestamp = i64::from_le_bytes(data[8..16].try_into().unwrap());

        Ok(ClientHello {
            obfuscated_integrity,
            obfuscated_timestamp,
        })
    }

    fn serialize(&self) -> std::io::Result<Vec<u8>> {
        use std::io::Write;

        let mut data = Vec::with_capacity(16);
        data.write_all(&self.obfuscated_integrity.to_le_bytes())?;
        data.write_all(&self.obfuscated_timestamp.to_le_bytes())?;

        Ok(data)
    }
}
