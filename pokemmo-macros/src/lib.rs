use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(Message, attributes(prefixed))]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Message can only be derived for structs with named fields"),
        },
        _ => panic!("Message can only be derived for structs"),
    };

    let mut serialize_statements = Vec::new();
    let mut deserialize_statements = Vec::new();
    let mut field_names = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        field_names.push(field_name);
        let field_type = &field.ty;

        // Check for #[prefixed(type)] attribute
        let prefixed_attr = field.attrs.iter().find(|attr| attr.path().is_ident("prefixed"));

        if let Some(attr) = prefixed_attr {
            // This is a Vec<u8> or String with a length prefix
            let prefix_type: Type = attr.parse_args().expect("Expected type in #[prefixed(type)]");
            
            serialize_statements.push(quote! {
                let size: #prefix_type = self.#field_name.len()
                    .try_into()
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
                data.write_all(&size.to_le_bytes())?;
                data.write_all(&self.#field_name)?;
            });

            deserialize_statements.push(quote! {
                let size = {
                    let size_bytes = std::mem::size_of::<#prefix_type>();
                    if rdr.position() as usize + size_bytes > data.len() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            concat!("Insufficient data for ", stringify!(#field_name), " size")
                        ));
                    }
                    let mut size_buf = vec![0u8; size_bytes];
                    rdr.read_exact(&mut size_buf)?;
                    let size_array: [u8; std::mem::size_of::<#prefix_type>()] = size_buf.try_into().unwrap();
                    #prefix_type::from_le_bytes(size_array) as usize
                };
                let mut #field_name = vec![0u8; size];
                rdr.read_exact(&mut #field_name)?;
            });
        } else {
            // Handle primitive types
            let type_str = quote!(#field_type).to_string();
            
            if type_str.contains("Vec") || type_str.contains("String") {
                panic!("Vec and String fields must have a #[prefixed(type)] attribute");
            }

            serialize_statements.push(quote! {
                data.write_all(&self.#field_name.to_le_bytes())?;
            });

            deserialize_statements.push(quote! {
                let #field_name = {
                    let size = std::mem::size_of::<#field_type>();
                    if rdr.position() as usize + size > data.len() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            concat!("Insufficient data for ", stringify!(#field_name))
                        ));
                    }
                    let mut buf = [0u8; std::mem::size_of::<#field_type>()];
                    rdr.read_exact(&mut buf)?;
                    #field_type::from_le_bytes(buf)
                };
            });
        }
    }

    let expanded = quote! {
        impl Message for #name {
            fn serialize(&self) -> std::io::Result<Vec<u8>> {
                use std::io::Write;
                let mut data = Vec::new();
                #(#serialize_statements)*
                Ok(data)
            }

            fn deserialize(data: &[u8]) -> std::io::Result<Self> {
                use std::io::Read;
                let mut rdr = std::io::Cursor::new(data);
                #(#deserialize_statements)*
                Ok(Self {
                    #(#field_names),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
