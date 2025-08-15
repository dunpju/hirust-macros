extern crate proc_macro;
mod auth_file;
mod delete;
mod empty_file;
mod gen_dist;
mod get;
mod head;
mod post;
mod put;
mod route_file;
mod scope;
mod tag;
mod utils;

use crate::route_file::route_file_impl;
use crate::tag::tag_impl;
use auth_file::auth_file_impl;
use empty_file::empty_file_impl;
use gen_dist::gen_dist_impl;
use proc_macro::TokenStream;
use scope::scope_impl;

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::{parse_quote, Attribute};

    // cargo test run -- --show-output
    #[test]
    fn run() {
        //let s = r#"#[post(path = "/post", tag = "controllers::test1::post", middleware = {middlewares::auth::auth}, desc = "post")]"#;
        let attr: Attribute = parse_quote! {
            //#[post(path = "/post", tag = "controllers::test1::post", middleware = {middlewares::auth::auth,middlewares::auth::auth1}, desc = "post" auth = false)]
            //#[post(path = "/login", tag = "LoginController.Login", desc = "登录")]
            #[head(path = "/head", tag = "crate::app::controllers::test::head", middleware = {middlewares::auth::my_auth_middleware}, desc = "head")]
        };
        println!("{:?}", attr);
        println!("{:?}", quote! {#attr});
        let auth_info = utils::parse_auth_info(quote! {#attr});
        println!("{:?}", auth_info);
    }
}

#[proc_macro_attribute]
pub fn empty_file(args: TokenStream, item: TokenStream) -> TokenStream {
    empty_file_impl(args, item)
}

#[proc_macro_attribute]
pub fn route_file(args: TokenStream, item: TokenStream) -> TokenStream {
    route_file_impl(args, item)
}

#[proc_macro_attribute]
pub fn auth_file(args: TokenStream, item: TokenStream) -> TokenStream {
    auth_file_impl(args, item)
}

#[proc_macro_attribute]
pub fn scope(args: TokenStream, item: TokenStream) -> TokenStream {
    scope_impl(args, item)
}

#[proc_macro_attribute]
pub fn gen_dist(args: TokenStream, item: TokenStream) -> TokenStream {
    gen_dist_impl(args, item)
}

#[proc_macro_attribute]
pub fn tag(args: TokenStream, item: TokenStream) -> TokenStream {
    tag_impl(args, item)
}

// https://rust.biofan.org/crates/attributes.html
#[proc_macro_attribute]
pub fn post(_args: TokenStream, item: TokenStream) -> TokenStream {
    //post_impl(args, item)
    item
}

#[proc_macro_attribute]
pub fn get(_args: TokenStream, item: TokenStream) -> TokenStream {
    //get_impl(args, item)
    item
}

#[proc_macro_attribute]
pub fn put(_args: TokenStream, item: TokenStream) -> TokenStream {
    //put_impl(args, item)
    item
}

#[proc_macro_attribute]
pub fn delete(_args: TokenStream, item: TokenStream) -> TokenStream {
    //delete_impl(args, item)
    item
}

#[proc_macro_attribute]
pub fn head(_args: TokenStream, item: TokenStream) -> TokenStream {
    //head_impl(args, item)
    item
}
