extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};

use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields};

use convert_case::{Case, Casing};

macro_rules! derive_error {
    ($string: tt) => {
        Error::new(Span::call_site(), $string)
            .to_compile_error()
            .into()
    };
}

#[proc_macro_derive(IntEnum)]
pub fn derive_int_enum(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let data_raw = &input.data;

    let data = if let Data::Enum(data) = data_raw {
        data
    } else {
        return derive_error!("IntEnum is only implemented for enums");
    };

    let repr: Ident = {
        let mut ident: Option<Ident> = None;
        for attr in input.attrs {
            if !attr.path.is_ident("repr") {
                continue;
            }
            ident = Some(attr.parse_args().unwrap());
            break;
        }
        ident.unwrap()
    };

    let mut generated = TokenStream2::new();
    let mut to_int_arms = TokenStream2::new();
    let mut str_to_int_arms = TokenStream2::new();

    for variant in &data.variants {
        let v_value = if let Some((_, val)) = &variant.discriminant {
            val
        } else {
            return derive_error!("Every IntEnum variant must have a defined disciminant");
        };

        let v_name = &variant.ident;
        let fields_in_variant = match &variant.fields {
            Fields::Unnamed(_) => quote_spanned! {variant.span()=> (..) },
            Fields::Unit => quote_spanned! { variant.span()=> },
            Fields::Named(_) => quote_spanned! {variant.span()=> {..} },
        };

        to_int_arms.extend(quote! {
            #name::#v_name #fields_in_variant => #v_value,
        });
        str_to_int_arms.extend(quote! {
            "#v_name" => #v_value,
        });
    }

    generated.extend(quote! {
        pub fn tag_to_int(&self) -> #repr {
            match self {
                #to_int_arms
            }
        }
        pub fn str_to_int(tag: &str) -> #repr {
            match tag {
                #str_to_int_arms
                _ => panic!("Tag should match one of the enum variants")
            }
        }
    });

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            // variant_checker_functions gets replaced by all the functions
            // that were constructed above
            #generated
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(UnwrapVariant)]
pub fn derive_unwrap_variant(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let data_raw = &input.data;

    let data = if let Data::Enum(data) = data_raw {
        data
    } else {
        return derive_error!("UnwrapVariant is only implemented for enums");
    };
    let mut generated = TokenStream2::new();

    for variant in &data.variants {
        let v_name = &variant.ident;
        let (ret, fields, ret_fields) = match &variant.fields {
            Fields::Unnamed(v) => {
                if v.unnamed.len() > 1 {
                    let mut field_names = vec![];
                    for i in 0..v.unnamed.len() {
                        field_names.push(format!("f{i}"));
                    }
                    let fields: TokenStream2 = format!("({})", field_names.join(",")).parse().unwrap();
                    (quote! { #v }, fields.clone(), fields)
                } else {
                    (
                        v.unnamed.first().unwrap().to_token_stream(),
                        quote! { (v) },
                        quote! { v },
                    )
                }
            }
            Fields::Unit => (quote! { () }, quote! {}, quote! { () }),
            Fields::Named(_) => return derive_error!("Named fields are unsupported"),
        };
        let snake_name = v_name.to_string().to_case(Case::Snake);
        let full_name = format_ident!("unwrap_{}", snake_name);

        generated.extend(quote! {
           pub fn  #full_name(self) -> Option<#ret> {
                if let #name::#v_name #fields = self {
                    Some(#ret_fields)
                } else {
                    None
                }
            }
        });
    }
    println!("{generated}");

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            // variant_checker_functions gets replaced by all the functions
            // that were constructed above
            #generated
        }
    };

    TokenStream::from(expanded)
}

