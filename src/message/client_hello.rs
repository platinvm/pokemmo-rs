#[derive(Debug, Clone)]
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

impl TryFrom<crate::packet::Payload> for ClientHello {
    type Error = std::io::Error;

    fn try_from(payload: crate::packet::Payload) -> Result<Self, Self::Error> {
        if payload.opcode != 0x00 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid opcode for ClientHello message",
            ));
        }

        if payload.data.len() != 16 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid data length for ClientHello message",
            ));
        }

        let obfuscated_integrity = i64::from_le_bytes(payload.data[0..8].try_into().unwrap());
        let obfuscated_timestamp = i64::from_le_bytes(payload.data[8..16].try_into().unwrap());

        Ok(ClientHello {
            obfuscated_integrity,
            obfuscated_timestamp,
        })
    }
}

impl TryInto<crate::packet::Payload> for ClientHello {
    type Error = std::io::Error;

    fn try_into(self) -> std::io::Result<crate::packet::Payload> {
        use std::io::Write;

        let mut data = Vec::new();
        data.write(&self.obfuscated_integrity.to_le_bytes())?;
        data.write(&self.obfuscated_timestamp.to_le_bytes())?;

        Ok(crate::packet::Payload { opcode: 0x00, data })
    }
}
