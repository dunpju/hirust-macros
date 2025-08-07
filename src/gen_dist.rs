use crate::utils;
use quote::quote;
use std::fs;
use std::path::Path;
use syn::{parse_macro_input, ItemFn};
use proc_macro::{TokenStream, TokenTree};

pub(crate) fn gen_dist_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut is_zip_filename = false;
    let mut zip_filename = String::new();
    let mut is_target_dir = false;
    let mut target_dir = String::new();
    for arg in args.into_iter() {
        if matches!(&arg, TokenTree::Ident(_)) && "zip".eq(&arg.to_string()) {
            is_zip_filename = true;
        }
        if is_zip_filename && matches!(&arg, TokenTree::Literal(_)) {
            let temp = arg.to_string();
            zip_filename = temp.clone().replace("\"", "");
            is_zip_filename = false;
        }
        if matches!(&arg, TokenTree::Ident(_)) && "target_dir".eq(&arg.to_string()) {
            is_target_dir = true;
        }
        if is_target_dir && matches!(&arg, TokenTree::Literal(_)) {
            let temp = arg.to_string();
            target_dir = temp.clone().replace("\"", "");
            is_target_dir = false;
        }
    }
    let filename = zip_filename.replace(".zip", "");
    if Path::new(&zip_filename).exists() {
        if Path::new(&filename).exists() {
            // 删除dist目录
            fs::remove_dir_all(&filename).expect(format!("{}清空失败", &filename).as_str());
        }
        println!("{}被清空", &filename);
    }
    // 解压dist.zip
    utils::extract_zip(&zip_filename, &target_dir).unwrap();
    println!("{}生成", filename);

    // 解析输入作为ItemFn类型，它是syn 提供的表示函数类型
    let input = parse_macro_input!(input as ItemFn);

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

    // 抽取函数体语句
    let statements = block.stmts;

    // 使用解析输入重构函数，然后输出
    quote!(
        // 在该函数上重复其他所有属性（保持不变）
        #(#attrs)*
        // 重构函数声明
        #vis #sig {

            // 创建新的块，即函数的主体部分，存储返回值作为变量，后续可返回给父函数
            let __result = {
                #(#statements)*
            };

            // 返回结果
            return __result;
        }
    )
    .into()
}
