use crate::utils;
use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use regex::Regex;
use std::fs;
use syn::{parse_file, parse_macro_input, parse_str, File, Item, ItemFn, Stmt};

pub(crate) fn scope_impl(args: TokenStream, input: TokenStream) -> TokenStream {
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

    let mut is_file = false;
    let mut file = String::new();
    for arg in args.into_iter() {
        if matches!(&arg, TokenTree::Ident(_)) && "file".eq(&arg.to_string()) {
            is_file = true;
        }
        if is_file && matches!(&arg, TokenTree::Literal(_)) {
            let temp = arg.to_string();
            file = temp.clone().replace("\"", "");
        }
    }

    let mut fn_name_list: Vec<String> = vec![];
    let mut scope_var_name = String::new();

    // 正则表达式模式
    let patterns =
        r"(\#\s*\[\s*post)|(\#\s*\[\s*macros\s*::\s*post)|(\#\s*\[\s*get)|(\#\s*\[\s*macros\s*::\s*get)|(\#\s*\[\s*put)|(\#\s*\[\s*macros\s*::\s*put)|(\#\s*\[\s*delete)|(\#\s*\[\s*macros\s*::\s*delete)|(\#\s*\[\s*head)|(\#\s*\[\s*macros\s*::\s*head)";
    let re = Regex::new(patterns).unwrap(); // 创建正则表达式对象

    let content = fs::read_to_string(file).expect("Should have been able to read the file");

    // 解析文件内容为 AST
    let syntax: File = parse_file(&content).expect("Not valid Rust code");

    // 寻找routes函数和定义的scope变量名
    for item in syntax.items {
        if let Item::Fn(ItemFn { sig, attrs, .. }) = item {
            let fn_name = &sig.ident;
            if fn_name.eq("routes") {
                // 函数tokens
                //let fn_sig_str = quote! {#sig}.to_string();
                // 提取函数参数
                let args_map = utils::parse_group_extract_args(quote! {#sig});
                // 参数名
                let cfg = args_map.values().next().unwrap();
                for statement in &statements {
                    // println!("表达式：{:?}", quote! {#statement});
                    let mut is_cfg_ident = false;
                    let statement_tokens = quote! {#statement};
                    for statement_token in statement_tokens {
                        match statement_token {
                            proc_macro2::TokenTree::Ident(ref ident) => {
                                if !cfg.eq(&ident.to_string()) && !is_cfg_ident {
                                    continue;
                                }
                                is_cfg_ident = true;
                            }
                            proc_macro2::TokenTree::Group(ref group) => {
                                if is_cfg_ident {
                                    let group_tokens = group.stream();
                                    for group_token in group_tokens {
                                        match group_token {
                                            proc_macro2::TokenTree::Ident(ref ident) => {
                                                scope_var_name = ident.to_string();
                                            }
                                            _ => (),
                                        }
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
            } else {
                for attr in &attrs {
                    println!("{}:{} {:?}", file!(), line!(), quote! {#attr}.to_string());
                    // 函数属性
                    if re.is_match(quote! {#attr}.to_string().as_str()) {
                        let auth_info = utils::parse_auth_info(quote! {#attr});
                        println!("{}:{} {:?}", file!(), line!(), auth_info);
                        fn_name_list.push(fn_name.to_string());
                        break;
                    }
                }
            }
        }
    }

    let mut new_statements: Vec<Stmt> = vec![];
    let mut i = 0;
    let statements_len = &statements.len();
    let scope_var_name = scope_var_name.replace("\"", "");

    // 组装scope
    for statement in &statements {
        new_statements.push(statement.clone());
        if i == statements_len - 2 {
            for fn_name in fn_name_list.clone() {
                let fn_name = fn_name.replace("\"", "");
                // 将字符串转换成Stmt
                let stmt = parse_str::<Stmt>(format!(
                    "let {} = {}.service({});",
                    //"let scope = scope.service(web::resource("/info").app_data(hirust_auth::Auth{ tag: "test".to_string(), desc: "desc".to_string(), middlewares: "middlewares::auth::my_auth_middleware".to_string(), auth: false}).wrap(from_fn(middlewares::auth::my_auth_middleware)).route(web::post().to(info)));",
                    scope_var_name, scope_var_name, fn_name
                ).as_str())
                .unwrap();
                new_statements.push(stmt.clone());
            }
        }
        i += 1;
    }

    // 使用解析输入重构函数，然后输出
    quote!(
        // 在该函数上重复其他所有属性（保持不变）
        #(#attrs)*
        // 重构函数声明
        #vis #sig {
            #(#new_statements)*
        }
    )
    .into()
}
