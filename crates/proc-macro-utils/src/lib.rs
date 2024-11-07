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
    container_name: Ident,
    src_map_name: Ident,
    fields: Punctuated<Either<HirPropField, HirDataField>, Token![,]>,
}

impl Parse for HirContainer {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        input.parse::<Token![struct]>()?;
        let container_name = input.parse()?;
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
        Ok(Self { attrs, vis, container_name, src_map_name, fields })
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
    access: Option<Punctuated<HirDataFieldAccess, Token![,]>>,
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

        let access = if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            let buffer;
            braced!(buffer in input);
            let idx_access = buffer.parse_terminated(HirDataFieldAccess::parse, Token![,])?;
            Some(idx_access)
        } else {
            None
        };

        Ok(Self { data_name, src_name, data_ty, data_id_ty, src_ty, access })
    }
}

struct HirDataFieldAccess {
    data_ty: Type,
    data_id_ty: Type,
    src_ty: Type,
}

impl Parse for HirDataFieldAccess {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let data_ty = input.parse()?;

        let buffer;
        bracketed!(buffer in input);
        let data_id_ty = buffer.parse()?;
        buffer.parse::<Token![|]>()?;
        let src_ty = buffer.parse()?;
        if !buffer.is_empty() {
            return Err(buffer.error("unexpected token"));
        }

        Ok(Self { data_ty, data_id_ty, src_ty })
    }
}

#[proc_macro]
pub fn define_container(input: TokenStream) -> TokenStream {
    let HirContainer { attrs, vis, container_name, src_map_name, fields, .. } =
        &parse_macro_input!(input as HirContainer);

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

    let cont_data_names = fields
        .iter()
        .filter_map(|field| field.as_ref().right())
        .map(|field| field.data_name.clone());

    let impl_arena = fields
        .iter()
        .filter_map(|field| field.as_ref().right())
        .flat_map(|HirDataField { data_name, data_ty, access: idx_access, data_id_ty, .. }| {
            let build = move |data_ty, data_id_ty| {
                quote! {
                    impl utils::get::GetRef<#data_id_ty> for #container_name {
                        type Output = #data_ty;

                        fn get(&self, idx: #data_id_ty) -> &Self::Output {
                            self.#data_name.get(idx)
                        }
                    }
                }
            };
            match idx_access {
                Some(access) => Either::Left(access.iter().map(
                    move |HirDataFieldAccess { data_ty, data_id_ty, .. }| {
                        build(data_ty, data_id_ty)
                    },
                )),
                None => Either::Right(iter::once(build(data_ty, data_id_ty.as_ref().unwrap()))),
            }
        });

    let data_def = quote! {
        #(#attrs)*
        #vis struct #container_name {
            #(#data_fields,)*
        }

        impl #container_name {
            pub fn shrink_to_fit(&mut self) {
                self.items.shrink_to_fit();
                #(self.#cont_data_names.shrink_to_fit();)*
            }
        }

        #(#impl_arena)*
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
        .flat_map(|HirDataField { src_name, src_ty, access: idx_access, data_id_ty, .. }| {
            let build = move |src_ty, data_id_ty| {
                quote! {
                    impl utils::get::Get<#src_ty> for #src_map_name {
                        type Output = #data_id_ty;

                        fn get(&self, src: #src_ty) -> Self::Output {
                            self.#src_name.get(src)
                        }
                    }

                    impl utils::get::Get<#data_id_ty> for #src_map_name {
                        type Output = #src_ty;
                        fn get(&self, idx: #data_id_ty) -> Self::Output {
                            self.#src_name.get(idx)
                        }
                    }
                }
            };
            match idx_access {
                Some(access) => access
                    .iter()
                    .map(move |HirDataFieldAccess { data_id_ty, src_ty, .. }| {
                        build(src_ty, data_id_ty)
                    })
                    .collect(),
                None => vec![build(src_ty, data_id_ty.as_ref().unwrap())],
            }
        });

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

        #(#impl_source_map)*
    };

    let output = quote! {
        #data_def
        #src_map_def
    };

    TokenStream::from(output)
}

struct HirContainerImpl {
    attrs: Vec<Attribute>,
    vis: Visibility,
    containers: Punctuated<(Type, Type), Token![,]>,
    access: Punctuated<HirDataFieldAccess, Token![,]>,
}

impl Parse for HirContainerImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        input.parse::<Token![enum]>()?;
        let buffer;
        braced!(buffer in input);
        let containers = buffer.parse_terminated(
            |buf| {
                let container = buf.parse()?;
                buf.parse::<Token![|]>()?;
                let src_map = buf.parse()?;
                Ok((container, src_map))
            },
            Token![,],
        )?;
        input.parse::<Token![=>]>()?;
        let buffer;
        braced!(buffer in input);
        let access = buffer.parse_terminated(HirDataFieldAccess::parse, Token![,])?;
        Ok(Self { attrs, vis, containers, access })
    }
}

#[proc_macro]
pub fn impl_container(input: TokenStream) -> TokenStream {
    let HirContainerImpl { attrs, vis, containers, access } =
        parse_macro_input!(input as HirContainerImpl);

    let container_def = containers.iter().map(|(name, _)| quote! { #name(Arc<#name>), });
    let container_srcmap_def = containers.iter().map(|(_, name)| quote! { #name(Arc<#name>), });

    let impls = access.iter().flat_map(|HirDataFieldAccess { data_ty, data_id_ty, src_ty }| {
        let data_arms = containers.iter().map(|(name, _)| {
            quote! { Self::#name(#name) => #name.get(idx), }
        });

        let src_arms = containers.iter().map(|(_, name)| {
            quote! { Self::#name(#name) => #name.get(src), }
        });

        let src_arms_2 = containers.iter().map(|(_, name)| {
            quote! { Self::#name(#name) => #name.get(idx), }
        });

        quote! {
            impl utils::get::GetRef<#data_id_ty> for Container {
                type Output = #data_ty;

                fn get(&self, idx: #data_id_ty) -> &Self::Output {
                    match self {
                        #(#data_arms)*
                    }
                }
            }

            impl utils::get::Get<#src_ty> for ContainerSrcMap {
                type Output = #data_id_ty;

                fn get(&self, src: #src_ty) -> Self::Output {
                    match self {
                        #(#src_arms)*
                    }
                }
            }

            impl utils::get::Get<#data_id_ty> for ContainerSrcMap {
                type Output = #src_ty;

                fn get(&self, idx: #data_id_ty) -> Self::Output {
                    match self {
                        #(#src_arms_2)*
                    }
                }
            }
        }
    });

    let output = quote! {
        utils::define_enum_deriving_from! {
            #(#attrs)*
            #vis enum Container {
                #(#container_def)*
            }
        }

        utils::define_enum_deriving_from! {
            #(#attrs)*
            #vis enum ContainerSrcMap {
                #(#container_srcmap_def)*
            }
        }

        #(#impls)*
    };
    TokenStream::from(output)
}
