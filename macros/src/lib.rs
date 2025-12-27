use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type, Expr, Lit};

// Maximum allowed size for prefixed fields to prevent DoS attacks
const MAX_PREFIXED_SIZE: usize = 10_485_760; // 10 MB

/// Derives serialization and deserialization for message payload types.
///
/// Implements the `Message` trait, automatically generating `serialize()` and `deserialize()` methods.
/// All fields are serialized in little-endian byte order. Variable-length fields must be annotated.
///
/// ## Supported Types
///
/// - **Integer types**: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`
///   - Serialized as little-endian bytes.
/// - **Vec<u8>**: Requires `#[prefixed(T)]` where `T` is an integer type for the length prefix.
///   - Format: `[length: T LE, data...]`
/// - **String**: Requires `#[prefixed(T)]` attribute (similar to Vec<u8>).
///
/// ## Attributes
///
/// - `#[prefixed(T)]`: Marks a `Vec` or `String` field with a length prefix type.
///   - Example: `#[prefixed(i16)]` prefixes the field with a 2-byte i16 length.
///
/// ## Examples
///
/// ```ignore
/// use pokemmo_macros::Message;
///
/// #[derive(Message)]
/// pub struct MyMessage {
///     version: u32,
///     count: i16,
///     #[prefixed(i16)]
///     payload: Vec<u8>,
/// }
///
/// let msg = MyMessage { version: 1, count: 10, payload: vec![1, 2, 3] };
/// let bytes = msg.serialize()?;
/// let decoded = MyMessage::deserialize(&bytes)?;
/// ```
///
/// ## Errors
///
/// The generated `deserialize()` method returns an error if:
/// - The input data is truncated (insufficient bytes).
/// - A prefixed length exceeds `MAX_PREFIXED_SIZE` (10 MB) to prevent DoS attacks.
///
/// ## Panic
///
/// The macro panics at compile time if:
/// - The struct contains tuple variants or unit variants.
/// - A `Vec` or `String` field lacks a `#[prefixed(T)]` attribute.
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
                    let size_array: [u8; std::mem::size_of::<#prefix_type>()] = size_buf
                        .try_into()
                        .map_err(|_| std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            concat!("Failed to parse size for ", stringify!(#field_name))
                        ))?;
                    let size_value = #prefix_type::from_le_bytes(size_array) as usize;
                    
                    // Validate size to prevent excessive memory allocation
                    const MAX_SIZE: usize = #MAX_PREFIXED_SIZE;
                    if size_value > MAX_SIZE {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            concat!("Field ", stringify!(#field_name), " size exceeds maximum allowed")
                        ));
                    }
                    
                    size_value
                };
                let mut #field_name = vec![0u8; size];
                rdr.read_exact(&mut #field_name)?;
            });
        } else {
            // Handle primitive types
            // Check if the type is Vec or String using proper type analysis
            let is_vec_or_string = match field_type {
                Type::Path(type_path) => {
                    if let Some(segment) = type_path.path.segments.last() {
                        let ident = &segment.ident;
                        ident == "Vec" || ident == "String"
                    } else {
                        false
                    }
                }
                _ => false,
            };
            
            if is_vec_or_string {
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

/// A procedural macro that implements the `Codec` trait for enums with opcode-based message routing.
///
/// Automatically generates `encode()` and `decode()` implementations that handle opcode-based
/// message serialization and deserialization. Each variant corresponds to a specific opcode,
/// and the macro generates the necessary match logic.
///
/// ## Requirements
///
/// - **Non-Unknown variants**: Must have an explicit opcode literal (e.g., `= 0x00u8`).
///   - Must be tuple variants with exactly one unnamed field.
///   - The field type must implement the `Message` trait.
/// - **Unknown variant** (optional): Must have named fields:
///   - `opcode`: Type `u8` or `i8` (encoded/decoded as a single little-endian byte).
///   - `data`: Type `Vec<u8>` carrying the raw payload.
///
/// ## Behavior
///
/// - **`encode()`**: Prepends the opcode to the serialized message payload.
///   - For known variants, the opcode is cast to `u8`.
///   - For `Unknown`, the opcode is encoded as a single LE byte (supports `u8` or `i8`).
/// - **`decode()`**: Reads the first byte as the opcode and dispatches to the appropriate variant.
///   - If the opcode matches a known variant, deserializes the payload via `Message::deserialize()`.
///   - Otherwise, falls back to `Unknown` (mapping the byte to the declared opcode type).
///
/// ## Examples
///
/// ```ignore
/// use pokemmo_macros::codec;
/// use pokemmo::message::Message;
///
/// #[codec]
/// pub enum LoginCodec {
///     Hello(HelloMessage) = 0x00u8,
///     Goodbye(GoodbyeMessage) = 0x01u8,
///     Unknown { opcode: i8, data: Vec<u8> },
/// }
///
/// let msg = LoginCodec::Hello(hello);
/// let encoded = msg.encode()?;
/// let decoded = LoginCodec::decode(&encoded)?;
/// ```
///
/// ## Generated Impls
///
/// In addition to `Codec`, the macro also generates:
/// - `Into<Codec> for MessageType` for each known variant.
/// - `TryFrom<Codec> for MessageType` for each known variant.
///
/// These enable ergonomic type conversion via `.into()` and `.try_into()`.
#[proc_macro_attribute]
pub fn codec(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    
    let enum_name = &input.ident;
    let vis = &input.vis;
    
    let data_enum = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return syn::Error::new_spanned(
                &input,
                "codec attribute can only be applied to enums"
            )
            .to_compile_error()
            .into();
        }
    };
    
    let mut variants_with_opcodes = Vec::new();
    // Track Unknown variant presence and whether its opcode is i8
    let mut has_unknown = false;
    let mut unknown_is_i8 = false;
    
    for variant in &data_enum.variants {
        let variant_name = &variant.ident;
        
        // Check if this is the Unknown variant
        if variant_name == "Unknown" {
            has_unknown = true;
            // Validate fields and detect opcode type
            match &variant.fields {
                Fields::Named(named) => {
                    for f in &named.named {
                        if let Some(ident) = &f.ident {
                            if ident == "opcode" {
                                if let Type::Path(tp) = &f.ty {
                                    if let Some(seg) = tp.path.segments.last() {
                                        let id = seg.ident.to_string();
                                        match id.as_str() {
                                            "i8" => unknown_is_i8 = true,
                                            "u8" => unknown_is_i8 = false,
                                            _ => {
                                                return syn::Error::new_spanned(
                                                    &f.ty,
                                                    "Unknown opcode field must be of type u8 or i8"
                                                ).to_compile_error().into();
                                            }
                                        }
                                    }
                                } else {
                                    return syn::Error::new_spanned(
                                        &f.ty,
                                        "Unknown opcode field must be a primitive integer type"
                                    ).to_compile_error().into();
                                }
                            }
                        }
                    }
                }
                _ => {
                    return syn::Error::new_spanned(
                        variant,
                        "Unknown variant must use named fields with an `opcode` and `data`"
                    ).to_compile_error().into();
                }
            }
            continue;
        }
        
        // Extract the opcode from the discriminant
        let opcode = match &variant.discriminant {
            Some((_, Expr::Lit(expr_lit))) => {
                match &expr_lit.lit {
                    Lit::Int(lit_int) => lit_int.clone(),
                    _ => {
                        return syn::Error::new_spanned(
                            variant,
                            "Expected integer literal for opcode"
                        )
                        .to_compile_error()
                        .into();
                    }
                }
            }
            _ => {
                return syn::Error::new_spanned(
                    variant,
                    "Each variant must have an explicit opcode (e.g., = 0x00u8)"
                )
                .to_compile_error()
                .into();
            }
        };
        
        // Extract the inner type from the tuple variant
        let inner_type = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                &fields.unnamed.first().unwrap().ty
            }
            _ => {
                return syn::Error::new_spanned(
                    variant,
                    "Each variant must have exactly one unnamed field"
                )
                .to_compile_error()
                .into();
            }
        };
        
        variants_with_opcodes.push((variant_name.clone(), opcode, inner_type.clone()));
    }
    
    // Generate the enum definition without discriminants
    let enum_variants = data_enum.variants.iter().map(|v| {
        let variant_name = &v.ident;
        let fields = &v.fields;
        quote! { #variant_name #fields }
    });
    
    // Generate encode match arms
    let mut all_encode_arms = variants_with_opcodes.iter().map(|(name, opcode, _)| {
        quote! {
            #enum_name::#name(msg) => {
                let mut msg_data = vec![#opcode as u8];
                msg_data.extend_from_slice(&msg.serialize()?);
                msg_data
            }
        }
    }).collect::<Vec<_>>();
    
    // Add unknown variant arm if present
    if has_unknown {
        let opcode_push = if unknown_is_i8 {
            quote! { i8::to_le_bytes(*opcode)[0] }
        } else {
            quote! { *opcode }
        };
        all_encode_arms.push(quote! {
            #enum_name::Unknown { opcode, data } => {
                let mut msg_data = Vec::with_capacity(1 + data.len());
                msg_data.push(#opcode_push);
                msg_data.extend_from_slice(data);
                msg_data
            }
        });
    }
    
    // Generate decode match arms
    let decode_arms = variants_with_opcodes.iter().map(|(name, opcode, inner_type)| {
        quote! {
            #opcode => Ok(#enum_name::#name(
                #inner_type::deserialize(&data[1..])?
            ))
        }
    });
    
    let decode_default_arm = if has_unknown {
        let opcode_expr = if unknown_is_i8 {
            quote! { i8::from_le_bytes([opcode]) }
        } else {
            quote! { opcode }
        };
        quote! {
            opcode => Ok(#enum_name::Unknown {
                opcode: #opcode_expr,
                data: data[1..].to_vec(),
            })
        }
    } else {
        quote! {
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unknown message opcode"
            ))
        }
    };
    
    // Generate Into implementations
    let into_impls = variants_with_opcodes.iter().map(|(name, _, inner_type)| {
        quote! {
            impl Into<#enum_name> for #inner_type {
                fn into(self) -> #enum_name {
                    #enum_name::#name(self)
                }
            }
        }
    });
    
    // Generate TryFrom implementations
    let try_from_impls = variants_with_opcodes.iter().map(|(name, _, inner_type)| {
        quote! {
            impl TryFrom<#enum_name> for #inner_type {
                type Error = ();
                
                fn try_from(value: #enum_name) -> Result<Self, Self::Error> {
                    match value {
                        #enum_name::#name(msg) => Ok(msg),
                        _ => Err(()),
                    }
                }
            }
        }
    });
    
    let expanded = quote! {
        #vis enum #enum_name {
            #(#enum_variants),*
        }
        
        impl super::Codec for #enum_name {
            fn encode(&self) -> std::io::Result<Vec<u8>> {
                use crate::message::Message;
                
                Ok(match self {
                    #(#all_encode_arms),*
                })
            }
            
            fn decode(data: &[u8]) -> std::io::Result<Self> {
                use crate::message::Message;
                
                if data.is_empty() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "No opcode found in message"
                    ));
                }
                
                match data[0] {
                    #(#decode_arms,)*
                    #decode_default_arm
                }
            }
        }
        
        #(#into_impls)*
        
        #(#try_from_impls)*
    };
    
    TokenStream::from(expanded)
}
