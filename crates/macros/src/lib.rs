//! Derive proc-macro  definitions for the `Mock` trait.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Fields, FieldsNamed, Meta, Token};

/// Derives `Mock` for both structs and enums if all their fields implement either `Mock` or
/// `Default`.
#[proc_macro_derive(Mock, attributes(mock, mock_default))]
pub fn derive_mock(token_stream: TokenStream) -> TokenStream {
    derive_mock_impl(token_stream)
}

fn derive_mock_impl(token_stream: TokenStream) -> TokenStream {
    let type_definition = parse_macro_input!(token_stream as DeriveInput);

    let identifier = type_definition.ident;

    let cfg_scope = match cfg_scope(type_definition.attrs) {
        Ok(scope) => scope,
        Err(err) => return err.into_compile_error().into(),
    };

    let self_definition_result = match type_definition.data {
        Data::Struct(data_struct) => derive_struct(data_struct),
        Data::Enum(data_enum) => derive_enum(data_enum),
        // TODO:
        Data::Union(data_union) => Err(syn::Error::new(data_union.union_token.span, "union types not supported")),
    };

    match self_definition_result {
        Ok(self_definition) => {
            quote! {
                #cfg_scope
                impl ::damock::Mock for #identifier {
                    fn mock() -> Self {
                        #self_definition
                    }
                }
            }
        }
        Err(err) => err.to_compile_error(),
    }
    .into()
}

fn cfg_scope(container_attributes: Vec<syn::Attribute>) -> syn::Result<Option<proc_macro2::TokenStream>> {
    // find a predicate inside matching the first cfg_attr(predicate, derive(Mock))
    // NOTE: method does not work if multiple Mock derives are used
    for attribute in container_attributes {
        if let syn::Meta::List(meta_list) = attribute.meta {
            if meta_list.path.is_ident("cfg_attr") {
                let mut nested_meta_iter = meta_list
                    .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
                    .into_iter();

                let predicate = nested_meta_iter.next().expect("cfg_attr must begin with a precidate");

                for nested_meta in nested_meta_iter {
                    if nested_meta.path().is_ident("derive") {
                        if let syn::Meta::List(nested_meta_list) = nested_meta {
                            for derive_meta in nested_meta_list.parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)? {
                                if derive_meta.is_ident("Mock") {
                                    return Ok(Some(quote! { #[cfg(#predicate)] }));
                                }
                            }
                        }
                    }
                }
            }
        };
    }

    Ok(None)
}

fn derive_struct(data_struct: syn::DataStruct) -> syn::Result<proc_macro2::TokenStream> {
    Ok(match data_struct.fields {
        Fields::Named(named_fields) => {
            let fields = fields::named(named_fields);

            quote! {
                Self {
                    #(#fields),*
                }
            }
        }
        Fields::Unnamed(tuple_fields) => {
            let fields = fields::tuple(tuple_fields);

            quote! { Self(#(#fields),*) }
        }
        Fields::Unit => quote! { Self },
    })
}

fn derive_enum(data_enum: syn::DataEnum) -> syn::Result<proc_macro2::TokenStream> {
    let mut variant_to_mock_iter = data_enum.variants.into_iter().filter_map(|variant| {
        variant
            .attrs
            .clone()
            .iter()
            .find(|attribute| match &attribute.meta {
                syn::Meta::Path(path) => path.is_ident("mock"),
                _ => false,
            })
            .map(|_| variant)
    });

    let Some(variant_to_mock) = variant_to_mock_iter.next() else {
        return Err(syn::Error::new(
            data_enum.enum_token.span,
            "no #[mock] attribute found in any of the listed variants",
        ));
    };

    if let Some(_another_variant_to_mock) = variant_to_mock_iter.next() {
        return Err(syn::Error::new(
            data_enum.enum_token.span,
            "expected only one #[mock] enum variant attribute, unable to infer which one to use.",
        ));
    }

    let variant_name = variant_to_mock.ident;

    Ok(match variant_to_mock.fields {
        Fields::Named(named_fields) => {
            let fields = fields::named(named_fields);

            quote! {
                Self::#variant_name {
                    #(#fields),*
                }
            }
        }
        Fields::Unnamed(tuple_fields) => {
            let fields = fields::tuple(tuple_fields);
            quote! {
                Self::#variant_name(#(#fields),*)
            }
        }
        Fields::Unit => {
            quote! {
                Self::#variant_name
            }
        }
    })
}

mod fields {
    use super::*;

    pub fn named(named_fields: FieldsNamed) -> impl Iterator<Item = proc_macro2::TokenStream> {
        named_fields
            .named
            .into_iter()
            .map(|field| {
                (
                    field.ident.expect("encountered named field without an identifier"),
                    mock_or_default(field.attrs),
                )
            })
            .map(|(field_name, mock_or_default)| quote! { #field_name: #mock_or_default })
    }

    pub fn tuple(tuple_fields: syn::FieldsUnnamed) -> impl Iterator<Item = proc_macro2::TokenStream> {
        tuple_fields.unnamed.into_iter().map(|field| mock_or_default(field.attrs))
    }

    fn mock_or_default(field_attributes: Vec<syn::Attribute>) -> proc_macro2::TokenStream {
        match field_attributes
            .into_iter()
            .any(|attribute| matches!(&attribute.meta, syn::Meta::Path(path) if path.is_ident("mock_default")))
        {
            true => quote! { Default::default() },
            false => quote! { ::damock::Mock::mock() },
        }
    }
}
