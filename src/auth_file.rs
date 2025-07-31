use super::utils;
use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use std::fs;
use syn::{parse_macro_input, ItemFn};

pub(crate) fn auth_file_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut is_filename = false;
    let mut filename = String::new();
    for arg in args.into_iter() {
        if matches!(&arg, TokenTree::Ident(_)) && "filename".eq(&arg.to_string()) {
            is_filename = true;
        }
        if is_filename && matches!(&arg, TokenTree::Literal(_)) {
            let temp = arg.to_string();
            filename = temp.clone().replace("\"", "");
            is_filename = false;
        }
    }

    // 删除文件
    if let Ok(_file) = fs::remove_file(&filename) {
        // 文件被删除
        println!("{}被删除", &filename.as_str())
    }

    // 清空文件
    // if let Ok(_file) = OpenOptions::new().write(true).truncate(true).open(&filename) {
    //     // 文件被截断为空
    //     println!("{}被清空", &filename.as_str())
    // }

    // 创建文件
    utils::create_file(filename.as_str());

    let temp_item = item.clone();

    // 解析输入作为ItemFn类型，它是syn 提供的表示函数类型
    let input = parse_macro_input!(item as ItemFn);

    let ItemFn {
        // 函数签名
        sig: _sig,
        // 函数可见性标识
        vis: _vis,
        // 函数体
        block,
        // 其他属性
        attrs: _attrs,
    } = input;

    // 抽取函数体语句
    let statements = block.stmts;

    // 使用解析输入重构函数，然后输出
    let temp_fn = quote!(
        #(#statements)*
    );

    let content = format!("{}", temp_fn);

    utils::write_file(filename.as_str(), &*("{".to_owned() + content.as_str() + "}"));
    temp_item
}