pub struct ContextedStream<S: std::io::Read + std::io::Write, C: crate::codec::Codec> {
    stream: S,
    _marker: std::marker::PhantomData<C>,
}

impl<T, C> std::ops::Deref for ContextedStream<T, C>
where
    T: std::io::Read + std::io::Write,
    C: crate::codec::Codec,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl<T, C> std::ops::DerefMut for ContextedStream<T, C>
where
    T: std::io::Read + std::io::Write,
    C: crate::codec::Codec,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

impl<S, C> ContextedStream<S, C>
where
    S: std::io::Read + std::io::Write,
    C: crate::codec::Codec,
{
    pub fn read_message<T: TryFrom<C>>(&mut self) -> std::io::Result<T> {
        let mut length_bytes = [0u8; 2];
        self.read_exact(&mut length_bytes)?;
        let length: usize = i16::from_le_bytes(length_bytes).try_into().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid message length")
        })?;

        let mut buffer = vec![0u8; length - 2];
        self.read_exact(&mut buffer)?;

        C::decode(&buffer)?.try_into().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to convert message")
        })
    }

    pub fn write_message(&mut self, message: impl Into<C>) -> std::io::Result<()> {
        let encoded = message.into().encode()?;
        let length: i16 = encoded.len().try_into().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Message too large")
        })?;
        self.write_all(&(length + 2).to_le_bytes())?;
        self.write_all(&encoded)?;
        Ok(())
    }
}

pub trait WithContext: std::io::Read + std::io::Write {
    fn with_context<C: crate::codec::Codec>(self) -> ContextedStream<Self, C>
    where
        Self: Sized,
    {
        ContextedStream {
            stream: self,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: std::io::Read + std::io::Write> WithContext for T {}
