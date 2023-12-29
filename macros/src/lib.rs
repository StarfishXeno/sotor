extern crate proc_macro;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Error, Fields};

macro_rules! derive_error {
    ($string: tt) => {
        Error::new(Span::call_site(), $string)
            .to_compile_error()
            .into()
    };
}

macro_rules! parse_enum {
    ($input:ident) => {{
        let input: DeriveInput = parse_macro_input!($input as DeriveInput);
        let name = input.ident.clone();
        let data = if let Data::Enum(data) = &input.data {
            data.clone()
        } else {
            return derive_error!("This macro is only implemented for enums");
        };

        (name, input, data)
    }};
}

macro_rules! get_enum_repr {
    ($input:ident) => {{
        let mut ident: Option<Ident> = None;
        for attr in &$input.attrs {
            if !attr.path.is_ident("repr") {
                continue;
            }
            ident = Some(attr.parse_args().unwrap());
            break;
        }

        if let Some(ident) = ident {
            ident
        } else {
            return derive_error!("Enum must have an int repr");
        }
    }};
}

#[proc_macro_derive(EnumToInt)]
pub fn derive_enum_to_int(input: TokenStream) -> TokenStream {
    let (name, input, data) = parse_enum!(input);
    let repr: Ident = get_enum_repr!(input);

    let mut to_int_arms = TokenStream2::new();
    let mut str_to_int_arms = TokenStream2::new();

    for variant in &data.variants {
        let Some((_, v_value)) = &variant.discriminant else {
            return derive_error!("Every IntEnum variant must have a defined disciminant");
        };

        let v_name = &variant.ident;
        let str_name = v_name.to_string();
        let fields_in_variant = match &variant.fields {
            Fields::Unnamed(_) => quote_spanned! {variant.span()=> (..) },
            Fields::Unit => quote_spanned! { variant.span()=> },
            Fields::Named(_) => quote_spanned! {variant.span()=> {..} },
        };

        to_int_arms.extend(quote! {
            #name::#v_name #fields_in_variant => #v_value,
        });
        str_to_int_arms.extend(quote! {
            #str_name => #v_value,
        });
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn to_int(&self) -> #repr {
                match self {
                    #to_int_arms
                }
            }
            pub fn str_to_int(variant: &'static str) -> #repr {
                match variant {
                    #str_to_int_arms
                    _ => panic!("Variant should match one of the enum variants")
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(EnumFromInt)]
pub fn derive_enum_from_int(input: TokenStream) -> TokenStream {
    let (name, input, data) = parse_enum!(input);
    let repr = get_enum_repr!(input);

    let mut match_arms = TokenStream2::new();

    for variant in &data.variants {
        let Some((_, v_value)) = &variant.discriminant else {
            return derive_error!("Every EnumFromInt variant must have a defined disciminant");
        };

        let v_name = &variant.ident;
        match &variant.fields {
            Fields::Unit => {}
            _ => continue,
        };

        match_arms.extend(quote! {
            #v_value => Ok(#name::#v_name),
        });
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics TryFrom<#repr> for #name #ty_generics #where_clause {
            type Error = ();

            fn try_from(id: #repr) -> Result<Self, Self::Error> {
                match id {
                    #match_arms
                    _ => Err(())
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(EnumList)]
pub fn derive_enum_list(input: TokenStream) -> TokenStream {
    let (name, input, data) = parse_enum!(input);

    let mut list = TokenStream2::new();
    let count = data.variants.len();

    for variant in &data.variants {
        let v_name = &variant.ident;
        let is_unit = matches!(&variant.fields, Fields::Unit);

        if !is_unit {
            return derive_error!("EnumList only works with unit enums");
        }
        list.extend(quote_spanned! {variant.span()=>
            #name::#v_name,
        });
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub const LIST: [#name; #count] = [#list];
            pub const COUNT: usize = #count;
        }

    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(EnumToString)]
pub fn derive_enum_to_string(input: TokenStream) -> TokenStream {
    let (name, input, data) = parse_enum!(input);

    let mut match_arms = TokenStream2::new();

    for variant in &data.variants {
        let v_name = &variant.ident;
        let str_name = v_name.to_string();
        let fields_in_variant = match &variant.fields {
            Fields::Unnamed(_) => quote_spanned! {variant.span()=> (..) },
            Fields::Unit => {
                quote_spanned! { variant.span()=> }
            }
            Fields::Named(_) => quote_spanned! {variant.span()=> {..} },
        };

        match_arms.extend(quote_spanned! {variant.span()=>
            #name::#v_name #fields_in_variant => #str_name,
        });
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    #match_arms
                })
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(UnwrapVariant)]
pub fn derive_unwrap_variant(input: TokenStream) -> TokenStream {
    let (name, input, data) = parse_enum!(input);
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
                    let fields: TokenStream2 =
                        format!("({})", field_names.join(",")).parse().unwrap();
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

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #generated
        }
    };

    TokenStream::from(expanded)
}
