#[derive(Default)]
pub struct Unknown(pub Vec<u8>);

crate::payload! {
    Unknown {
        const OPCODE: i8 = i8::MIN;
        type Context = ();

        fn serialize(&self, _ctx: &Self::Context) -> std::io::Result<Vec<u8>> {
            Ok(self.0.clone())
        }

        fn deserialize(data: &[u8], _ctx: &Self::Context) -> std::io::Result<Self> {
            Ok(Unknown(data.to_vec()))
        }
    }
}
