pub struct Packet {
    size: i16,
    data: Vec<u8>,
}

impl Packet {
    pub fn new(
        payload: Payload,
        encryption: Encryption,
        checksum: Checksum,
    ) -> std::io::Result<Self> {
        use std::io::Write;

        let mut data = Vec::new();
        data.write(&payload.opcode.to_le_bytes())?;
        data.write(&payload.data)?;

        match encryption {
            Encryption::None => {}
        }

        match checksum {
            Checksum::None => {}
        }

        let size: i16 = data
            .len()
            .try_into()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

        let size = size + 2;

        Ok(Self { size, data })
    }

    pub fn payload(&self, encryption: Encryption, checksum: Checksum) -> Payload {
        match encryption {
            Encryption::None => {}
        }

        match checksum {
            Checksum::None => {}
        }

        Payload {
            opcode: i8::from_le_bytes([self.data[0]]),
            data: self.data[1..].to_vec(),
        }
    }
}

pub trait ReadPacket: std::io::Read {
    fn read_packet(&mut self) -> std::io::Result<Packet> {
        let mut size_buf = [0u8; 2];
        self.read_exact(&mut size_buf)?;
        let size = i16::from_le_bytes(size_buf) - 2;

        let capacity = size
            .try_into()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

        let mut data = vec![0u8; capacity];
        self.read_exact(&mut data)?;

        Ok(Packet { size, data })
    }
}

impl<T: std::io::Read> ReadPacket for T {}

pub trait WritePacket: std::io::Write {
    fn write_packet(&mut self, packet: &Packet) -> std::io::Result<()> {
        self.write_all(&packet.size.to_le_bytes())?;
        self.write_all(&packet.data)?;
        Ok(())
    }
}

impl<T: std::io::Write> WritePacket for T {}

pub enum Checksum {
    None,
}

pub enum Encryption {
    None,
}

pub struct Payload {
    pub opcode: i8,
    pub data: Vec<u8>,
}
