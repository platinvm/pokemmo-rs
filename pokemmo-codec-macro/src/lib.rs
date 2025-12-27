use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Expr, Lit};

/// A procedural macro that implements the Codec trait for enums with explicit opcodes.
///
/// # Example
/// ```ignore
/// #[codec]
/// pub enum MyCodec {
///     VariantA(crate::message::VariantA) = 0x00u8,
///     VariantB(crate::message::VariantB) = 0x01u8,
///     Unknown{opcode: u8, data: Vec<u8>},
/// }
/// ```
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
    let mut unknown_variant = None;
    
    for variant in &data_enum.variants {
        let variant_name = &variant.ident;
        
        // Check if this is the Unknown variant
        if variant_name == "Unknown" {
            unknown_variant = Some(variant_name.clone());
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
                let mut msg_data = vec![#opcode];
                msg_data.extend_from_slice(&msg.serialize()?);
                msg_data
            }
        }
    }).collect::<Vec<_>>();
    
    // Add unknown variant arm if present
    if unknown_variant.is_some() {
        all_encode_arms.push(quote! {
            #enum_name::Unknown { opcode, data } => {
                let mut msg_data = vec![*opcode];
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
    
    let decode_default_arm = if unknown_variant.is_some() {
        quote! {
            opcode => Ok(#enum_name::Unknown {
                opcode,
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
