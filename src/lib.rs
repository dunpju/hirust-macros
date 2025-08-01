extern crate proc_macro;
mod empty_file;
mod auth_file;
mod utils;
mod scope;
mod gen_dist;
mod post;
mod tag;
mod get;
mod put;
mod delete;
mod head;
mod route_file;

use auth_file::auth_file_impl;
use empty_file::empty_file_impl;
use gen_dist::gen_dist_impl;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_str, ItemFn};
use scope::scope_impl;
use crate::post::post_impl;
use crate::get::get_impl;
use crate::put::put_impl;
use crate::delete::delete_impl;
use crate::head::head_impl;
use crate::route_file::route_file_impl;
use crate::tag::tag_impl;


#[proc_macro_attribute]
pub fn empty_file(args: TokenStream, item: TokenStream) -> TokenStream {
    empty_file_impl(args, item)
}

#[proc_macro_attribute]
pub fn router_file(args: TokenStream, item: TokenStream) -> TokenStream {
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
pub fn scope_unfold(_args: TokenStream, item: TokenStream) -> TokenStream {

    let input = parse_macro_input!(item as ItemFn);

    let ItemFn {
        // 函数签名
        sig,
        // 函数可见性标识
        vis,
        // 函数体
        block,
        // 其他属性
        attrs,
    } = input;


    //let contents = include_str!("../.././resources/scope").to_string();
    let contents = String::new();

    println!("contents {}", contents);

    // 将字符串解析为Rust代码片段
    let handler_expr = parse_str::<syn::Expr>(contents.as_str()).unwrap();

    // 抽取函数体语句
    let statements = block.stmts;

    quote!(
            // 在该函数上重复其他所有属性（保持不变）
            #(#attrs)*
            // 重构函数声明
            #vis #sig {
                #handler_expr;
                // 创建新的块，即函数的主体部分，存储返回值作为变量，后续可返回给父函数
                let __result = {
                    #(#statements)*
                };
                // 返回结果
                return __result;
            }
        ).into()
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
pub fn post(args: TokenStream, item: TokenStream) -> TokenStream {
    post_impl(args, item)
}

#[proc_macro_attribute]
pub fn get(args: TokenStream, item: TokenStream) -> TokenStream {
    get_impl(args, item)
}

#[proc_macro_attribute]
pub fn put(args: TokenStream, item: TokenStream) -> TokenStream {
    put_impl(args, item)
}

#[proc_macro_attribute]
pub fn delete(args: TokenStream, item: TokenStream) -> TokenStream {
    delete_impl(args, item)
}

#[proc_macro_attribute]
pub fn head(args: TokenStream, item: TokenStream) -> TokenStream {
    head_impl(args, item)
}