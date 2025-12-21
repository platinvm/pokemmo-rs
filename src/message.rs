mod client_hello;
mod client_ready;
mod server_hello;

pub use client_hello::ClientHello;
pub use client_ready::ClientReady;
pub use server_hello::{Checksum, ServerHello};

#[derive(Debug, Clone)]
pub enum Message {
    ClientHello {
        obfuscated_integrity: i64,
        obfuscated_timestamp: i64,
    },
    ServerHello {
        public_key: Vec<u8>,
        signature: Vec<u8>,
        checksum_size: i8,
    },
    ClientReady {
        public_key: Vec<u8>,
    },
    Unknown {
        opcode: i8,
        data: Vec<u8>,
    },
}

impl Into<Vec<u8>> for Message {
    fn into(self) -> Vec<u8> {
        use std::io::Write;
        let mut data = Vec::new();
        match self {
            Message::ClientHello {
                obfuscated_integrity,
                obfuscated_timestamp,
            } => {
                data.write_all(&0x00i8.to_le_bytes()).unwrap();
                data.write_all(&obfuscated_integrity.to_le_bytes()).unwrap();
                data.write_all(&obfuscated_timestamp.to_le_bytes()).unwrap();
            }
            Message::ServerHello {
                public_key,
                signature,
                checksum_size,
            } => {
                data.write_all(&0x01i8.to_le_bytes()).unwrap();
                let pk_size: i16 = public_key.len() as i16;
                data.write_all(&pk_size.to_le_bytes()).unwrap();
                data.write_all(&public_key).unwrap();
                let sig_size: i16 = signature.len() as i16;
                data.write_all(&sig_size.to_le_bytes()).unwrap();
                data.write_all(&signature).unwrap();
                data.write_all(&[checksum_size as u8]).unwrap();
            }
            Message::ClientReady { public_key } => {
                data.write_all(&0x02i8.to_le_bytes()).unwrap();
                let size: i16 = public_key.len() as i16;
                data.write_all(&size.to_le_bytes()).unwrap();
                data.write_all(&public_key).unwrap();
            }
            Message::Unknown {
                opcode,
                data: payload,
            } => {
                data.write_all(&opcode.to_le_bytes()).unwrap();
                data.write_all(&payload).unwrap();
            }
        }
        data
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = std::io::Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        use std::io::Cursor;
        use std::io::Read;

        let mut rdr = Cursor::new(data);
        let mut opcode_buf = [0u8; 1];
        rdr.read_exact(&mut opcode_buf)?;
        let opcode = i8::from_le_bytes(opcode_buf);

        match opcode {
            0 => {
                let mut buf = [0u8; 8];
                rdr.read_exact(&mut buf)?;
                let obfuscated_integrity = i64::from_le_bytes(buf);
                rdr.read_exact(&mut buf)?;
                let obfuscated_timestamp = i64::from_le_bytes(buf);
                Ok(Message::ClientHello {
                    obfuscated_integrity,
                    obfuscated_timestamp,
                })
            }
            0x01 => {
                let mut size_buf = [0u8; 2];
                rdr.read_exact(&mut size_buf)?;
                let pk_size = i16::from_le_bytes(size_buf) as usize;
                let mut public_key = vec![0u8; pk_size];
                rdr.read_exact(&mut public_key)?;
                rdr.read_exact(&mut size_buf)?;
                let sig_size = i16::from_le_bytes(size_buf) as usize;
                let mut signature = vec![0u8; sig_size];
                rdr.read_exact(&mut signature)?;
                let mut checksum_buf = [0u8; 1];
                rdr.read_exact(&mut checksum_buf)?;
                let checksum_size = checksum_buf[0] as i8;
                Ok(Message::ServerHello {
                    public_key,
                    signature,
                    checksum_size,
                })
            }
            0x02 => {
                let mut size_buf = [0u8; 2];
                rdr.read_exact(&mut size_buf)?;
                let size = i16::from_le_bytes(size_buf) as usize;
                let mut public_key = vec![0u8; size];
                rdr.read_exact(&mut public_key)?;
                Ok(Message::ClientReady { public_key })
            }
            _ => {
                let mut payload = Vec::new();
                rdr.read_to_end(&mut payload)?;
                Ok(Message::Unknown {
                    opcode,
                    data: payload,
                })
            }
        }
    }
}

pub trait WriteMessage: std::io::Write {
    fn write_message<T: Into<Message>>(&mut self, message: T) -> std::io::Result<()> {
        let msg: Message = message.into();
        let bytes: Vec<u8> = msg.into();
        let size = (bytes.len() + 2) as i16;
        self.write_all(&size.to_le_bytes())?;
        self.write_all(&bytes)?;
        Ok(())
    }
}

impl<T: std::io::Write> WriteMessage for T {}

pub trait ReadMessage: std::io::Read {
    fn read_message<T: TryFrom<Message>>(&mut self) -> std::io::Result<T> {
        let mut size_buf = [0u8; 2];
        self.read_exact(&mut size_buf)?;
        let size = i16::from_le_bytes(size_buf) as usize;
        if size < 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid message size",
            ));
        }
        let mut msg_buf = vec![0u8; size - 2];
        self.read_exact(&mut msg_buf)?;
        let msg = Message::try_from(msg_buf.as_slice())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid message"))?;
        T::try_from(msg).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Type conversion failed")
        })
    }
}

impl<T: std::io::Read> ReadMessage for T {}
