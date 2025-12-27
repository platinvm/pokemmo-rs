use pokemmo_macros::Message;

// Define the Message trait for testing
pub trait Message: Sized {
    fn serialize(&self) -> std::io::Result<Vec<u8>>;
    fn deserialize(data: &[u8]) -> std::io::Result<Self>;
}

#[derive(Message)]
pub struct SimpleMessage {
    field1: u32,
    field2: i64,
}

#[derive(Message)]
pub struct MessageWithVec {
    value: i32,
    #[prefixed(i16)]
    data: Vec<u8>,
}

#[derive(Message)]
pub struct ComplexMessage {
    a: i64,
    b: i64,
    #[prefixed(i16)]
    vec1: Vec<u8>,
    #[prefixed(i16)]
    vec2: Vec<u8>,
    c: i8,
}

#[test]
fn test_simple_message_serialize() {
    let msg = SimpleMessage {
        field1: 42,
        field2: -100,
    };
    
    let result = msg.serialize().unwrap();
    
    // u32 (42) = 4 bytes + i64 (-100) = 8 bytes = 12 bytes total
    assert_eq!(result.len(), 12);
    
    // Check field1 (u32 = 42 in little endian)
    assert_eq!(&result[0..4], &42u32.to_le_bytes());
    
    // Check field2 (i64 = -100 in little endian)
    assert_eq!(&result[4..12], &(-100i64).to_le_bytes());
}

#[test]
fn test_simple_message_deserialize() {
    let mut data = Vec::new();
    data.extend_from_slice(&42u32.to_le_bytes());
    data.extend_from_slice(&(-100i64).to_le_bytes());
    
    let msg = SimpleMessage::deserialize(&data).unwrap();
    
    assert_eq!(msg.field1, 42);
    assert_eq!(msg.field2, -100);
}

#[test]
fn test_message_with_vec_serialize() {
    let msg = MessageWithVec {
        value: 123,
        data: vec![1, 2, 3, 4, 5],
    };
    
    let result = msg.serialize().unwrap();
    
    // i32 (4 bytes) + i16 length (2 bytes) + 5 bytes data = 11 bytes
    assert_eq!(result.len(), 11);
    
    // Check value
    assert_eq!(&result[0..4], &123i32.to_le_bytes());
    
    // Check length prefix
    assert_eq!(&result[4..6], &5i16.to_le_bytes());
    
    // Check data
    assert_eq!(&result[6..11], &[1, 2, 3, 4, 5]);
}

#[test]
fn test_message_with_vec_deserialize() {
    let mut data = Vec::new();
    data.extend_from_slice(&123i32.to_le_bytes());
    data.extend_from_slice(&5i16.to_le_bytes());
    data.extend_from_slice(&[1, 2, 3, 4, 5]);
    
    let msg = MessageWithVec::deserialize(&data).unwrap();
    
    assert_eq!(msg.value, 123);
    assert_eq!(msg.data, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_complex_message_roundtrip() {
    let original = ComplexMessage {
        a: 1000,
        b: -2000,
        vec1: vec![10, 20, 30],
        vec2: vec![40, 50],
        c: -5,
    };
    
    let serialized = original.serialize().unwrap();
    let deserialized = ComplexMessage::deserialize(&serialized).unwrap();
    
    assert_eq!(deserialized.a, 1000);
    assert_eq!(deserialized.b, -2000);
    assert_eq!(deserialized.vec1, vec![10, 20, 30]);
    assert_eq!(deserialized.vec2, vec![40, 50]);
    assert_eq!(deserialized.c, -5);
}

#[test]
fn test_deserialize_insufficient_data() {
    let data = vec![1, 2, 3]; // Too short for SimpleMessage (needs 12 bytes)
    
    let result = SimpleMessage::deserialize(&data);
    
    assert!(result.is_err());
}

#[test]
fn test_empty_vec() {
    let msg = MessageWithVec {
        value: 456,
        data: vec![],
    };
    
    let serialized = msg.serialize().unwrap();
    let deserialized = MessageWithVec::deserialize(&serialized).unwrap();
    
    assert_eq!(deserialized.value, 456);
    assert_eq!(deserialized.data, vec![]);
}
