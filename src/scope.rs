use crate::route_file::route_cfg;
use crate::utils;
use crate::utils::create_and_append;
use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use syn::{File, Item, ItemFn, Stmt, parse_file, parse_macro_input, parse_str};

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
    let mut auth_info_map: HashMap<String, hirust_auth::Auth> = HashMap::new();
    let mut scope_var_name = String::new();

    // 正则表达式模式
    let patterns = r"(\#\s*\[\s*post)|(\#\s*\[\s*macros\s*::\s*post)|(\#\s*\[\s*get)|(\#\s*\[\s*macros\s*::\s*get)|(\#\s*\[\s*put)|(\#\s*\[\s*macros\s*::\s*put)|(\#\s*\[\s*delete)|(\#\s*\[\s*macros\s*::\s*delete)|(\#\s*\[\s*head)|(\#\s*\[\s*macros\s*::\s*head)";
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
                    // 函数属性
                    if re.is_match(quote! {#attr}.to_string().as_str()) {
                        //println!("{}:{} {:?}", file!(), line!(), quote! {#attr}.to_string());
                        let auth_info = utils::parse_auth_info(quote! {#attr});
                        // println!("{}:{} {:?}", file!(), line!(), auth_info);

                        let tag = auth_info.clone().tag.clone();
                        match hirust_auth::exist(tag.clone()) {
                            Some(_) => {
                                panic!("This handler tag: {} is duplication", tag.clone());
                            }
                            None => {
                                let route_cfg = route_cfg();
                                if route_cfg.is_empty() {
                                    panic!(
                                        "file: {}, line: {}, message: route config is empty, please check the route configuration path and compilation order.",
                                        file!(),
                                        line!()
                                    );
                                }
                                let serialized = serde_json::to_string(&auth_info.clone()).unwrap();
                                create_and_append(route_cfg.as_str(), &serialized.as_str());
                            }
                        }
                        auth_info_map.insert(fn_name.clone().to_string(), auth_info.clone());
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
                let auth_info = auth_info_map.get(&fn_name).unwrap();
                //println!("{}:{} {:?}", file!(), line!(), &auth_info.clone());
                let auth_info_serialized = serde_json::to_string(&auth_info.clone())
                    .expect("auth_info serialization failed");

                if auth_info.clone().middleware.is_empty() {
                    let format_str = format!(
                        "let {} = {}.service(web::resource(\"{}\").app_data(r#\"{}\"#.to_string()).route(web::{}().to({})));",
                        scope_var_name,
                        scope_var_name,
                        auth_info.path,
                        auth_info_serialized,
                        auth_info.method,
                        fn_name
                    );
                    // 将字符串转换成Stmt
                    let stmt = parse_str::<Stmt>(format_str.as_str()).unwrap();
                    new_statements.push(stmt.clone());
                } else {
                    let middlewares: Vec<String> = auth_info
                        .clone()
                        .middleware
                        .clone()
                        .split(",")
                        .map(|m| m.to_string().replace(" ", ""))
                        .collect();

                    let mut format_str = format!(
                        "let {} = {}.service(web::resource(\"{}\").app_data(r#\"{}\"#.to_string())",
                        scope_var_name, scope_var_name, auth_info.path, auth_info_serialized
                    );

                    let mut wrap_str = String::new();
                    for middleware in middlewares {
                        let temp = format!(".wrap(from_fn({}))", middleware);
                        wrap_str += &temp;
                    }
                    format_str += &wrap_str;

                    let route_str =
                        format!(".route(web::{}().to({})));", auth_info.method, fn_name);
                    format_str += &route_str;

                    // 将字符串转换成Stmt
                    let stmt = parse_str::<Stmt>(format_str.as_str()).unwrap();
                    new_statements.push(stmt.clone());
                }
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
