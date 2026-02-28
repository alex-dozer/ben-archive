mod ben_enum;
mod bitspec;
mod blot;
mod lucius;
mod schema;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

use crate::lucius::derive_lucius;

#[proc_macro_derive(BenSchema, attributes(bschema))]
pub fn ben_schema(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    match schema::derive_ben_schema(ast) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(BenEnum, attributes(benum))]
pub fn ben_enum(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    match ben_enum::derive_ben_enum(ast) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(Bitspec, attributes(bspec))]
pub fn bitspec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match bitspec::expand_bitspec(&input) {
        Ok(ts) => ts.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Lucius, attributes(lspec))]
pub fn lucius(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match lucius::derive_lucius(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(Blot, attributes(blotspec))]
pub fn blot(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match blot::derive_blot(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
