extern crate proc_macro;

use std::iter;

use either::Either;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Ident, Token, Type, Visibility, braced, bracketed, parse::Parse, parse_macro_input,
    punctuated::Punctuated,
};

struct HirContainer {
    attrs: Vec<Attribute>,
    vis: Visibility,
    data_name: Ident,
    src_map_name: Ident,
    fields: Punctuated<Either<HirPropField, HirDataField>, Token![,]>,
}

impl Parse for HirContainer {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        input.parse::<Token![struct]>()?;
        let data_name = input.parse()?;
        input.parse::<Token![|]>()?;
        let src_map_name = input.parse()?;

        let content;
        braced!(content in input);
        let fields = content.parse_terminated(
            |input| {
                if input.peek2(Token![:]) {
                    input.parse::<HirPropField>().map(Either::Left)
                } else {
                    input.parse::<HirDataField>().map(Either::Right)
                }
            },
            Token![,],
        )?;
        Ok(Self { attrs, vis, data_name, src_map_name, fields })
    }
}

struct HirPropField {
    name: Ident,
    ty: Type,
}

impl Parse for HirPropField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(Self { name, ty })
    }
}

struct HirDataField {
    data_name: Ident,
    src_name: Ident,
    data_ty: Type,
    data_id_ty: Option<Type>,
    src_ty: Type,
    idx_access: Option<Punctuated<HirDataFieldAccess, Token![,]>>,
}

impl Parse for HirDataField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let data_name = input.parse()?;
        input.parse::<Token![|]>()?;
        let src_name = input.parse()?;

        input.parse::<Token![:]>()?;

        let data_ty = input.parse::<Type>()?;

        let buffer;
        bracketed!(buffer in input);
        let data_id_ty = if buffer.peek(Token![_]) {
            buffer.parse::<Token![_]>()?;
            None
        } else {
            Some(buffer.parse()?)
        };
        buffer.parse::<Token![|]>()?;
        let src_ty = buffer.parse()?;
        if !buffer.is_empty() {
            return Err(buffer.error("unexpected token"));
        }

        let idx_access = if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            let buffer;
            braced!(buffer in input);
            let idx_access = buffer.parse_terminated(HirDataFieldAccess::parse, Token![,])?;
            Some(idx_access)
        } else {
            None
        };

        Ok(Self { data_name, src_name, data_ty, data_id_ty, src_ty, idx_access })
    }
}

struct HirDataFieldAccess {
    data_ty: Type,
    data_id_ty: Type,
    delegate: bool,
    src_ty: Type,
}

impl Parse for HirDataFieldAccess {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let data_ty = input.parse()?;

        let buffer;
        bracketed!(buffer in input);
        let data_id_ty = buffer.parse()?;
        let delegate = if buffer.peek(Token![|]) {
            buffer.parse::<Token![|]>()?;
            false
        } else {
            buffer.parse::<Token![=>]>()?;
            true
        };
        let src_ty = buffer.parse()?;
        if !buffer.is_empty() {
            return Err(buffer.error("unexpected token"));
        }

        Ok(Self { data_ty, data_id_ty, delegate, src_ty })
    }
}

#[proc_macro]
pub fn define_hir_container_data(input: TokenStream) -> TokenStream {
    let HirContainer { attrs, vis, data_name, src_map_name, fields, .. } =
        parse_macro_input!(input as HirContainer);

    // Generate the fields for the data struct
    let data_fields = fields.iter().map(|field| match field {
        Either::Left(HirPropField { name, ty }) => {
            quote! { #vis #name: #ty }
        }
        Either::Right(HirDataField { data_name, data_ty, data_id_ty, .. }) => {
            if data_id_ty.is_some() {
                quote! { #vis #data_name: Arena<#data_ty> }
            } else {
                quote! { #vis #data_name: #data_ty }
            }
        }
    });

    if !fields
        .iter()
        .any(|field| field.as_ref().left().map(|field| field.name == "items").unwrap_or(false))
    {
        panic!("missing 'items' field");
    }

    let cont_data_names =
        fields.iter().filter_map(|field| Some(field.as_ref().right()?.data_name.clone()));

    let impl_arena = fields
        .iter()
        .filter_map(|field| field.as_ref().right())
        .map(|HirDataField { data_name, data_ty, idx_access, .. }| {
            let Some(access) = idx_access else {
                return Either::Right(iter::once(quote! { #data_name[#data_ty], }));
            };

            let res = access.iter().map(
                move |HirDataFieldAccess { data_ty, delegate, data_id_ty, .. }| {
                    if *delegate {
                        quote! { #data_name[#data_id_ty => #data_ty], }
                    } else {
                        quote! { #data_name[#data_ty], }
                    }
                },
            );

            Either::Left(res)
        })
        .flatten();

    let data_def = quote! {
        #(#attrs)*
        #vis struct #data_name {
            #(#data_fields,)*
        }

        impl #data_name {
            pub fn shrink_to_fit(&mut self) {
                self.items.shrink_to_fit();
                #(self.#cont_data_names.shrink_to_fit();)*
            }
        }

        impl_arena_idx! { #data_name =>
            #(#impl_arena)*
        }
    };

    let src_fields = fields.iter().filter_map(|field| field.as_ref().right()).map(
        |HirDataField { src_name, data_id_ty, data_ty, src_ty, .. }| {
            if data_id_ty.is_some() {
                quote! { #vis #src_name: SourceMap<#src_ty, #data_ty> }
            } else {
                quote! { #vis #src_name: #src_ty }
            }
        },
    );

    let cont_src_names = fields
        .iter()
        .filter_map(|field| field.as_ref().right())
        .map(|field| field.src_name.clone());

    let impl_source_map = fields
        .iter()
        .filter_map(|field| field.as_ref().right())
        .map(|HirDataField { src_name, src_ty, idx_access, data_id_ty, .. }| match idx_access {
            Some(access) => access
                .iter()
                .map(move |HirDataFieldAccess { data_id_ty, src_ty, .. }| {
                    quote! { #src_name[#src_ty, #data_id_ty], }
                })
                .collect::<Vec<_>>(),
            None => vec![quote! { #src_name[#src_ty, #data_id_ty], }],
        })
        .flatten();

    let src_map_def = quote! {
        #(#attrs)*
        #vis struct #src_map_name {
            #(#src_fields,)*
        }

        impl #src_map_name {
            pub fn shrink_to_fit(&mut self) {
                #(self.#cont_src_names.shrink_to_fit();)*
            }
        }

        impl_source_map_idx! { #src_map_name =>
            #(#impl_source_map)*
        }
    };

    let output = quote! {
        #data_def
        #src_map_def
    };

    TokenStream::from(output)
}
