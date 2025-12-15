#[derive(Default)]
pub struct Unknown(pub Vec<u8>);

crate::payload! {
    Unknown {
        const OPCODE: i8 = i8::MIN;
        type Context = ();
        type Error = std::io::Error;

        fn serialize(&self, _ctx: &Self::Context) -> Result<Vec<u8>, Self::Error> {
            Ok(self.0.clone())
        }

        fn deserialize(data: &[u8], _ctx: &Self::Context) -> Result<Self, Self::Error> {
            Ok(Unknown(data.to_vec()))
        }
    }
}
