use db_code_macro_lib::db_meta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(DbDao)]
pub fn db(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let mut meta = db_meta::DbMeta::get().lock().expect("db_meta::DbMeta::get().lock()");
    (*meta).push(&ast);
    TokenStream::from(quote! {})
}

#[proc_macro_derive(DbSub)]
pub fn db_sub(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let mut meta = db_meta::DbMeta::get().lock().expect("db_meta::DbMeta::get().lock()");
    (*meta).push_sub_struct(&ast);
    TokenStream::from(quote! {})
}
