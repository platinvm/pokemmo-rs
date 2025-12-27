# pokemmo-macros

This crate provides derive macros for the pokemmo-rs library.

## Message Derive Macro

The `Message` derive macro automatically implements the `Message` trait for structs, providing serialization and deserialization functionality with minimal boilerplate.

### Supported Types

- **Integer types**: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`
  - Serialized as little-endian bytes
- **Vec<u8>**: Requires `#[prefixed(T)]` attribute where `T` is an integer type for the length prefix

### Usage

```rust
use pokemmo_rs::Message;

#[derive(Message)]
pub struct MyMessage {
    field1: u32,
    field2: i64,
    #[prefixed(i16)]  // Vec fields must specify a length prefix type
    field3: Vec<u8>,
}
```

This generates implementations for:
- `fn serialize(&self) -> std::io::Result<Vec<u8>>`
- `fn deserialize(data: &[u8]) -> std::io::Result<Self>`

### Example

```rust
let msg = MyMessage {
    field1: 42,
    field2: -100,
    field3: vec![1, 2, 3, 4, 5],
};

// Serialize
let bytes = msg.serialize()?;

// Deserialize
let decoded = MyMessage::deserialize(&bytes)?;
```

### Implementation Details

- All integer types are serialized in little-endian byte order
- Vec fields are prefixed with their length, encoded as the specified integer type
- The macro validates that Vec and String fields have the `#[prefixed(T)]` attribute
- Deserialization includes bounds checking to prevent buffer overruns

## `#[codec]` Enum Macro

Implements the `Codec` trait for enums that represent message opcodes and payloads.

### Rules

- Non-`Unknown` variants must have an explicit opcode literal (e.g., `= 0x00u8`).
- Each non-`Unknown` variant must be a tuple variant with exactly one unnamed field containing a type that implements `Message`.
- The enum’s `Unknown` variant is optional. When present, it must be a struct variant with named fields:
  - `opcode`: may be `u8` or `i8` (encoded/decoded as a little-endian single byte)
  - `data`: `Vec<u8>` carrying the raw payload

### Example

```rust
#[codec]
pub enum Login {
    ClientHello(crate::message::ClientHello) = 0x00u8,
    ServerHello(crate::message::ServerHello) = 0x01u8,
    ClientReady(crate::message::ClientReady) = 0x02u8,
    // Unknown uses its own opcode (i8) and raw data
    Unknown { opcode: i8, data: Vec<u8> },
}
```

### Behavior

- `encode()`: prefixes the payload with the variant opcode; casts explicit opcodes to `u8`. For `Unknown`, encodes the `opcode` field as a single LE byte (supporting `u8` or `i8`).
- `decode()`: reads the first byte as the opcode, matches known opcodes to deserialize the payload via `Message::deserialize`, and falls back to `Unknown` by mapping the opcode into the declared type (`u8` or `i8`).
