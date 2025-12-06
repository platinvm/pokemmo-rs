use std::io::{Read, Write};

/// A wrapper around a stream that logs all read and write operations.
pub struct LoggingStream<T> {
    inner: T,
}

impl<T> LoggingStream<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    fn format_hex(data: &[u8]) -> String {
        let mut result = String::new();
        for (i, chunk) in data.chunks(16).enumerate() {
            result.push_str(&format!("{:04x}  ", i * 16));
            
            // Hex bytes
            for (j, byte) in chunk.iter().enumerate() {
                if j == 8 {
                    result.push(' ');
                }
                result.push_str(&format!("{:02x} ", byte));
            }
            
            // Padding for incomplete lines
            for j in chunk.len()..16 {
                if j == 8 {
                    result.push(' ');
                }
                result.push_str("   ");
            }
            
            // ASCII representation
            result.push_str(" |");
            for byte in chunk {
                let ch = if *byte >= 0x20 && *byte <= 0x7e {
                    *byte as char
                } else {
                    '.'
                };
                result.push(ch);
            }
            result.push('|');
            
            if i < data.chunks(16).len() - 1 {
                result.push('\n');
            }
        }
        result
    }
}

impl<T: Read> Read for LoggingStream<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        if n > 0 {
            println!("[READ] {} bytes:", n);
            println!("{}", Self::format_hex(&buf[..n]));
        }
        Ok(n)
    }
}

impl<T: Write> Write for LoggingStream<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        println!("[WRITE] {} bytes:", buf.len());
        println!("{}", Self::format_hex(buf));
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        println!("[FLUSH]");
        self.inner.flush()
    }
}
