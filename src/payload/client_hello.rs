use std::time::SystemTime;

pub struct Context {
    pub primary_obfuscation_value: i64,
    pub secondary_obfuscation_value: i64,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            primary_obfuscation_value: 3214621489648854472,
            secondary_obfuscation_value: -4214651440992349575,
        }
    }
}

pub struct ClientHello {
    integrity: i64,
    timestamp: SystemTime,
}

impl ClientHello {
    pub fn new(integrity: i64, timestamp: SystemTime) -> Self {
        Self {
            integrity,
            timestamp,
        }
    }

    pub fn integrity(&self) -> i64 {
        self.integrity
    }

    pub fn timestamp(&self) -> &SystemTime {
        &self.timestamp
    }
}

impl Default for ClientHello {
    fn default() -> Self {
        Self {
            integrity: rand::random(),
            timestamp: SystemTime::now(),
        }
    }
}

crate::payload! {
    ClientHello {
        const OPCODE: i8 = 0x00;
        type Context = Context;
        type Error = std::io::Error;

        fn serialize(&self, ctx: &Self::Context) -> Result<Vec<u8>, Self::Error> {
            let mut data = Vec::new();

            let integrity_obfuscated = self.integrity ^ ctx.primary_obfuscation_value;

            let timestamp_millis = self
                .timestamp
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;

            let timestamp_obfuscated =
                timestamp_millis ^ self.integrity ^ ctx.secondary_obfuscation_value;

            data.extend_from_slice(&integrity_obfuscated.to_le_bytes());
            data.extend_from_slice(&timestamp_obfuscated.to_le_bytes());

            Ok(data)
        }

        fn deserialize(data: &[u8], ctx: &Self::Context) -> Result<Self, Self::Error> {
            if data.len() < 16 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Not enough data for ClientHello",
                ));
            }

            let integrity_obfuscated = i64::from_le_bytes(data[0..8].try_into().unwrap());
            let timestamp_obfuscated = i64::from_le_bytes(data[8..16].try_into().unwrap());

            let integrity = integrity_obfuscated ^ ctx.primary_obfuscation_value;
            let timestamp_millis = timestamp_obfuscated ^ integrity ^ ctx.secondary_obfuscation_value;

            let timestamp =
                SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(timestamp_millis as u64);

            Ok(Self {
                integrity,
                timestamp,
            })
        }
    }
}
