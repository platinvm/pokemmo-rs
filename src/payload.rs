pub mod client_hello;
pub mod client_ready;
pub mod server_hello;
pub mod unknown;

pub trait Payload: Sized + Default {
    const OPCODE: i8;

    type Context: Default;

    fn serialize(&self, ctx: &Self::Context) -> std::io::Result<Vec<u8>>;
    fn deserialize(data: &[u8], ctx: &Self::Context) -> std::io::Result<Self>;
}

/// This macro lets you implement the [`Payload`] trait as well as generate roundtrip tests.
#[macro_export]
macro_rules! payload {
    (
        $ty:ident {
            const OPCODE: i8 = $opcode:expr;
            type Context = $ctx:ty;

            fn serialize $serialize_sig:tt -> std::io::Result<Vec<u8>> $serialize:block
            fn deserialize $deserialize_sig:tt -> std::io::Result<Self> $deserialize:block
        }
    ) => {
        impl $crate::payload::Payload for $ty {
            const OPCODE: i8 = $opcode;

            type Context = $ctx;

            fn serialize $serialize_sig -> std::io::Result<Vec<u8>> $serialize
            fn deserialize $deserialize_sig -> std::io::Result<Self> $deserialize
        }

        paste::paste! {
            #[cfg(test)]
            mod [<roundtrip_test_ $ty:snake>] {
                use super::*;
                use $crate::payload::Payload;

                #[test]
                fn roundtrip() {
                    let v = $ty::default();
                    let ctx = <$ty as Payload>::Context::default();
                    let data = v.serialize(&ctx).expect("serialize failed");
                    let v2 = <$ty>::deserialize(&data, &ctx).expect("deserialize failed");
                    assert_eq!(data, v2.serialize(&ctx).expect("re-serialize failed"));
                }
            }
        }
    };
}
