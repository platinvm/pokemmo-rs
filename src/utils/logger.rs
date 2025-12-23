use std::io::{self, Read, Write};

pub struct Logger<T: Read + Write> {
    inner: T,
}

impl<T: Read + Write> Logger<T> {
    pub fn new(inner: T) -> Self {
        Logger { inner }
    }
    fn hexdump(prefix: &str, buf: &[u8]) {
        print!("{} ({} bytes):\n", prefix, buf.len());
        for (i, chunk) in buf.chunks(16).enumerate() {
            print!("{:04x}  ", i * 16);
            for (j, byte) in chunk.iter().enumerate() {
                if j == 8 {
                    print!(" ");
                }
                print!("{:02x} ", byte);
            }
            for j in chunk.len()..16 {
                if j == 8 {
                    print!(" ");
                }
                print!("   ");
            }
            print!(" |");
            for byte in chunk {
                let ch = if *byte >= 0x20 && *byte <= 0x7e {
                    *byte as char
                } else {
                    '.'
                };
                print!("{}", ch);
            }
            println!("|");
        }
    }
    pub fn info(&self, msg: &str) {
        println!("[INFO]: {}", msg);
    }
}

impl<T: Read + Write> Read for Logger<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        Self::hexdump("Read", &buf[..n]);
        Ok(n)
    }
}

impl<T: Read + Write> Write for Logger<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        Self::hexdump("Write", &buf[..n]);
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        let res = self.inner.flush();
        println!("Flush");
        res
    }
}
