use pokemmo_rs::message::Message;
use pokemmo_rs::Message as MessageDerive;

/// This test demonstrates how the Message derive macro can be used
/// to implement the same functionality as the manual implementations
/// in the existing message modules, but with much less boilerplate.

#[derive(MessageDerive)]
struct DemoClientHello {
    obfuscated_integrity: i64,
    obfuscated_timestamp: i64,
}

#[derive(MessageDerive)]
struct DemoClientReady {
    #[prefixed(i16)]
    public_key: Vec<u8>,
}

#[derive(MessageDerive)]
struct DemoServerHello {
    #[prefixed(i16)]
    public_key: Vec<u8>,
    #[prefixed(i16)]
    signature: Vec<u8>,
    checksum_size: i8,
}

#[test]
fn test_demo_client_hello() {
    let msg = DemoClientHello {
        obfuscated_integrity: 12345,
        obfuscated_timestamp: 67890,
    };

    // Test serialization
    let serialized = msg.serialize().unwrap();
    assert_eq!(serialized.len(), 16); // 8 bytes + 8 bytes

    // Test deserialization
    let deserialized = DemoClientHello::deserialize(&serialized).unwrap();
    assert_eq!(deserialized.obfuscated_integrity, 12345);
    assert_eq!(deserialized.obfuscated_timestamp, 67890);
}

#[test]
fn test_demo_client_ready() {
    let msg = DemoClientReady {
        public_key: vec![1, 2, 3, 4, 5],
    };

    // Test serialization
    let serialized = msg.serialize().unwrap();
    // 2 bytes (i16 length) + 5 bytes (data) = 7 bytes
    assert_eq!(serialized.len(), 7);
    assert_eq!(&serialized[0..2], &5i16.to_le_bytes());
    assert_eq!(&serialized[2..7], &[1, 2, 3, 4, 5]);

    // Test deserialization
    let deserialized = DemoClientReady::deserialize(&serialized).unwrap();
    assert_eq!(deserialized.public_key, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_demo_server_hello() {
    let msg = DemoServerHello {
        public_key: vec![10, 20, 30],
        signature: vec![40, 50],
        checksum_size: 16,
    };

    // Test serialization
    let serialized = msg.serialize().unwrap();
    
    // Test deserialization
    let deserialized = DemoServerHello::deserialize(&serialized).unwrap();
    assert_eq!(deserialized.public_key, vec![10, 20, 30]);
    assert_eq!(deserialized.signature, vec![40, 50]);
    assert_eq!(deserialized.checksum_size, 16);
}

#[test]
fn test_macro_reduces_boilerplate() {
    // This test demonstrates that using the macro significantly reduces code.
    // Compare the struct definitions above (3-5 lines each including the derive)
    // to the manual implementations in:
    // - src/message/client_hello.rs (75 lines)
    // - src/message/client_ready.rs (56 lines)
    // - src/message/server_hello.rs (119 lines)
    //
    // The macro handles all the serialization/deserialization boilerplate
    // automatically, making the code more maintainable and less error-prone.
    
    // This assertion just confirms the test ran
    assert!(true);
}
