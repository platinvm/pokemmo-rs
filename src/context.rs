/// A connection context for reading and writing codec messages over a stream.
///
/// `ContextedStream` wraps a stream (typically a `TcpStream`) and provides high-level methods
/// for reading and writing typed messages with automatic encoding and decoding.
/// It enforces the pokemmo-spec length-prefixed framing protocol.
///
/// Framing per spec:
/// - Handshake phase (unencrypted): `Length (i16 LE) || Packet`
/// - Secure phase (encrypted): `Length (i16 LE) || Encrypted Data || Checksum`
///
/// Note: This crate currently demonstrates the handshake message flow; encryption and
/// checksums are documented in the spec but not implemented here.
///
/// ## Type Parameters
///
/// - `S`: The underlying stream type (must implement `Read` and `Write`).
/// - `C`: The codec type defining which message variants are supported.
///
/// ## Examples
///
/// ```ignore
/// use pokemmo::context::WithContext;
/// use pokemmo::codec::Login;
/// use std::net::TcpStream;
///
/// let stream = TcpStream::connect("127.0.0.1:2106")?;
/// let mut ctx = stream.with_context::<Login>();
/// ctx.write_message(client_hello)?;
/// let msg: ClientHello = ctx.read_message()?;
/// ```
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
    /// Reads a single codec message from the stream and converts it to the target type.
    ///
    /// The message is expected to be framed as: `[length: i16 LE, payload...]` where
    /// `length` includes the 2-byte length prefix itself. The payload is decoded as
    /// codec type `C` and then converted to the target type `T` via `TryFrom`.
    ///
    /// ## Type Parameters
    ///
    /// - `T`: The target message type, must be convertible from the codec type `C`.
    ///
    /// ## Errors
    ///
    /// Returns an error if:
    /// - Reading from the stream fails (I/O error).
    /// - The length field is invalid or negative.
    /// - The codec decode fails (unknown opcode, malformed data).
    /// - The conversion from codec to target type fails.
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

    /// Writes a message to the stream with length-prefixed framing.
    ///
    /// Encodes the message using the codec and prefixes it with a 2-byte little-endian
    /// length field (including the length field itself). The message is then written to the stream.
    ///
    /// ## Type Parameters
    ///
    /// - The message must be convertible to the codec type `C` via `Into`.
    ///
    /// ## Errors
    ///
    /// Returns an error if:
    /// - Writing to the stream fails (I/O error).
    /// - The encoded message exceeds the maximum representable length (32767 bytes).
    /// - Codec encoding fails.
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
    /// Wraps this stream in a `ContextedStream` with the specified codec type.
    ///
    /// # Type Parameters
    ///
    /// - `C`: The codec type defining which message variants are supported.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use pokemmo::context::WithContext;
    /// use pokemmo::codec::Login;
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:2106")?;
    /// let mut ctx = stream.with_context::<Login>();
    /// ```
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
