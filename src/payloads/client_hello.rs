use crate::{Context, Payload};
use std::time::SystemTime;

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

impl Payload for ClientHello {
    const OPCODE: i8 = 0x00;

    fn encode(&self, mut data: impl std::io::Write, ctx: &Context) -> Result<(), std::io::Error> {
        let integrity_obfuscated = self.integrity ^ ctx.primary_obfuscation_value;

        let timestamp_millis = self
            .timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let timestamp_obfuscated =
            timestamp_millis ^ self.integrity ^ ctx.secondary_obfuscation_value;

        data.write_all(&integrity_obfuscated.to_le_bytes())?;
        data.write_all(&timestamp_obfuscated.to_le_bytes())?;

        Ok(())
    }

    fn decode(data: impl std::io::Read, ctx: &Context) -> Result<Self, std::io::Error> {
        let mut integrity_buf = [0u8; 8];
        let mut timestamp_buf = [0u8; 8];

        let mut reader = data;
        reader.read_exact(&mut integrity_buf)?;
        reader.read_exact(&mut timestamp_buf)?;

        let integrity_obfuscated = i64::from_le_bytes(integrity_buf);
        let timestamp_obfuscated = i64::from_le_bytes(timestamp_buf);

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
