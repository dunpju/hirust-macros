use crate::route_file::route_cfg;
use hirust_auth;
use proc_macro::{TokenStream, TokenTree};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use std::path::Path;
use zip::ZipArchive;

#[allow(dead_code)]
pub fn create_file(file_path: &str) {
    // 检查文件是否存在
    if !Path::new(file_path).exists() {
        // 文件不存在，尝试创建文件
        match std::fs::File::create(file_path) {
            Ok(_) => println!("文件创建成功：{}", file_path),
            Err(e) => println!("创建文件失败：{}", e),
        }
    }
}

#[allow(dead_code)]
pub fn create_and_append(file_path: &str, content: &str) {
    // 创建文件
    create_file(file_path);

    // 打开文件并追加内容
    match OpenOptions::new().append(true).open(file_path) {
        Ok(mut file) => {
            // 追加内容
            if let Err(e) = writeln!(file, "{}", content) {
                println!("追加内容失败：{}", e);
            }
        }
        Err(e) => println!("打开文件失败：{}", e),
    }
}

#[allow(dead_code)]
pub fn write_file(file_path: &str, content: &str) {
    match OpenOptions::new().write(true).open(file_path) {
        Ok(mut file) => {
            // 追加内容
            if let Err(e) = writeln!(file, "{}", content) {
                println!("写文件内容失败：{}", e);
            }
        }
        Err(e) => println!("打开文件失败：{}", e),
    }
}

#[allow(dead_code)]
pub fn reverse_string(s: &str) -> String {
    s.chars().rev().collect::<String>()
}

/**
let zip_path = "example.zip";
let extract_dir = "extracted_files";
*/
#[allow(dead_code)]
pub fn extract_zip(zip_path: &str, extract_to: &str) -> io::Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file_in_zip = archive.by_index(i)?;
        let outpath = match file_in_zip.enclosed_name() {
            Some(path) => {
                let mut path_buf = Path::new(extract_to).to_path_buf();
                path_buf.push(path);
                path_buf
            }
            None => continue,
        };

        if file_in_zip.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file_in_zip, &mut outfile)?;
        }
    }
    Ok(())
}

#[allow(unused)]
pub fn parse_token(args: TokenStream, req_map: HashMap<String, String>) -> (String, String) {
    let auth_info = parse_auth_info(proc_macro2::TokenStream::from(args));

    let path = auth_info.clone().path.clone();
    let tag = auth_info.clone().tag.clone();

    let middlewares: Vec<String> = auth_info
        .clone()
        .middlewares
        .clone()
        .split(",")
        .map(|m| m.to_string().replace(" ", ""))
        .collect();

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

    let mut contents = String::new();
    if !middlewares.is_empty() {
        let mut req = String::new();

        if req_map.contains_key("actix_web::HttpRequest") {
            req = "&".to_owned() + &*req_map.get("actix_web::HttpRequest").unwrap().to_string();
        } else if req_map.contains_key("HttpRequest") {
            req = "&".to_owned() + &*req_map.get("HttpRequest").unwrap().to_string();
        } else if req_map.contains_key("&HttpRequest") {
            req = req_map.get("&HttpRequest").unwrap().to_string();
        } else {
            panic!(
                "There is no request parameter {} `actix_web::HttpRequest`",
                req
            );
        }

        for middleware in middlewares {
            // 调用拦截器
            let temp = format!(
                r#"
                match interceptor({}({}, {})) {{
                    Some(response) => return response.respond_to({}),
                    _ => (),
                }}
                "#,
                middleware,
                req.clone().to_string(),
                tag.clone().to_string(),
                req.clone().to_string()
            );
            contents += &temp;
        }

        contents = format!(r#"{{{}}}"#, contents);
    } else {
        contents = format!(r#"{{{}}}"#, "");
    }

    (path, contents)
}

#[allow(unused)]
pub fn parse_attr(args: TokenStream) -> hirust_auth::Auth {
    let mut is_method = false;
    let mut method = String::new();
    let mut is_path = false;
    let mut path = String::new();
    let mut is_middleware = false;
    let mut middlewares: Vec<String> = vec![];
    let mut middleware = String::new();
    let mut is_tag = false;
    let mut tag = String::new();
    let mut is_auth = false;
    let mut auth = true;
    let mut is_desc = false;
    let mut desc = String::new();
    for arg in args.into_iter() {
        println!("{}:{} {:?}", file!(), line!(), &arg);
        if matches!(&arg, TokenTree::Ident(_)) && "method".eq(&arg.to_string()) {
            is_method = true;
        }
        if is_path && matches!(&arg, TokenTree::Literal(_)) {
            let temp = arg.to_string();
            method = temp.clone().replace("\"", "");
            is_method = false;
        }
        if matches!(&arg, TokenTree::Ident(_)) && "path".eq(&arg.to_string()) {
            is_path = true;
        }
        if is_path && matches!(&arg, TokenTree::Literal(_)) {
            let temp = arg.to_string();
            path = temp.clone().replace("\"", "");
            is_path = false;
        }
        if matches!(&arg, TokenTree::Ident(_)) && "middleware".eq(&arg.to_string()) {
            is_middleware = true;
        }
        if is_middleware && matches!(&arg, TokenTree::Group(_)) {
            middleware = arg.to_string();
            middleware = middleware
                .clone()
                .replace("{", "")
                .replace("}", "")
                .replace(" ", "");
            middlewares = middleware
                .split(",")
                .map(|m| m.to_string().replace(" ", ""))
                .collect();
            is_middleware = false;
        }
        if matches!(&arg, TokenTree::Ident(_)) && "tag".eq(&arg.to_string()) {
            is_tag = true;
        }
        if is_tag && matches!(&arg, TokenTree::Literal(_)) {
            tag = arg.clone().to_string();
            is_tag = false
        }
        if matches!(&arg, TokenTree::Ident(_)) && "auth".eq(&arg.to_string()) {
            is_auth = true;
        }
        if is_auth && !"auth".eq(&arg.to_string()) && matches!(&arg, TokenTree::Ident(_)) {
            if "false".eq(&arg.to_string()) {
                auth = false;
            }
            is_auth = false;
        }
        if matches!(&arg, TokenTree::Ident(_)) && "desc".eq(&arg.to_string()) {
            is_desc = true;
        }
        if is_desc && matches!(&arg, TokenTree::Literal(_)) {
            let temp = arg.to_string(); // rust 如何把代码块里的字符串拿到代码块外面来
            desc = temp.replace("\"", "");
            is_desc = false;
        }
    }

    hirust_auth::Auth {
        method: method.clone().replace("\"", ""),
        path: path.clone().replace("\"", ""),
        tag: tag.clone().replace("\"", ""),
        desc: desc.clone().replace("\"", ""),
        middlewares: middleware.clone().replace("\"", ""),
        auth,
    }
    .clone()
}

#[allow(unused)]
pub fn parse_auth_info(args: proc_macro2::TokenStream) -> hirust_auth::Auth {
    let mut is_method = false;
    let mut method = String::new();
    let mut is_path = false;
    let mut path = String::new();
    let mut is_middleware = false;
    let mut middlewares: Vec<String> = vec![];
    let mut middleware = String::new();
    let mut is_tag = false;
    let mut tag = String::new();
    let mut is_auth = false;
    let mut auth = true;
    let mut is_desc = false;
    let mut desc = String::new();
    for arg in args.into_iter() {
        match arg {
            // 遍历TokenTree::Group下的TokenStream
            proc_macro2::TokenTree::Group(ref group) => {
                println!("{}:{} {:?}", file!(), line!(), &group);
                // 获取组内的TokenStream并再次遍历
                let group_tokens = group.stream();
                for inner_group in group_tokens {
                    match inner_group {
                        // ref 模式 https://rustwiki.org/zh-CN/rust-by-example/scope/borrow/ref.html
                        proc_macro2::TokenTree::Ident(ref ident) => {
                            println!("{}:{} {:?}", file!(), line!(), &ident);
                            method = ident.clone().to_string().replace("\"", "");
                            println!("{}:{} {}", file!(), line!(), method);
                        }
                        proc_macro2::TokenTree::Group(ref group) => {
                            println!("{}:{} {:?}", file!(), line!(), &group);
                            // 获取组内的TokenStream并再次遍历
                            let group_tokens = group.stream();
                            for inner_group in group_tokens {
                                match inner_group {
                                    proc_macro2::TokenTree::Ident(ref ident) => {
                                        println!("{}:{} {}", file!(), line!(), &ident.to_string());
                                    }
                                    proc_macro2::TokenTree::Literal(ref literal) => {
                                        println!("{}:{} {}", file!(), line!(), &literal.to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        if matches!(&arg, proc_macro2::TokenTree::Ident(_)) && "path".eq(&arg.to_string()) {
            is_path = true;
        }
        if is_path && matches!(&arg, proc_macro2::TokenTree::Literal(_)) {
            let temp = arg.to_string();
            path = temp.clone().replace("\"", "");
            is_path = false;
        }
        if matches!(&arg, proc_macro2::TokenTree::Ident(_)) && "middleware".eq(&arg.to_string()) {
            is_middleware = true;
        }
        if is_middleware && matches!(&arg, proc_macro2::TokenTree::Group(_)) {
            middleware = arg.to_string();
            middleware = middleware
                .clone()
                .replace("{", "")
                .replace("}", "")
                .replace(" ", "");
            middlewares = middleware
                .split(",")
                .map(|m| m.to_string().replace(" ", ""))
                .collect();
            is_middleware = false;
        }
        if matches!(&arg, proc_macro2::TokenTree::Ident(_)) && "tag".eq(&arg.to_string()) {
            is_tag = true;
        }
        if is_tag && matches!(&arg, proc_macro2::TokenTree::Literal(_)) {
            tag = arg.clone().to_string();
            is_tag = false
        }
        if matches!(&arg, proc_macro2::TokenTree::Ident(_)) && "auth".eq(&arg.to_string()) {
            is_auth = true;
        }
        if is_auth
            && !"auth".eq(&arg.to_string())
            && matches!(&arg, proc_macro2::TokenTree::Ident(_))
        {
            if "false".eq(&arg.to_string()) {
                auth = false;
            }
            is_auth = false;
        }
        if matches!(&arg, proc_macro2::TokenTree::Ident(_)) && "desc".eq(&arg.to_string()) {
            is_desc = true;
        }
        if is_desc && matches!(&arg, proc_macro2::TokenTree::Literal(_)) {
            let temp = arg.to_string(); // rust 如何把代码块里的字符串拿到代码块外面来
            desc = temp.replace("\"", "");
            is_desc = false;
        }
    }

    let auth_info = hirust_auth::Auth {
        method: method.clone().replace("\"", ""),
        path: path.clone().replace("\"", ""),
        tag: tag.clone().replace("\"", ""),
        desc: desc.clone().replace("\"", ""),
        middlewares: middleware.clone().replace("\"", ""),
        auth,
    };

    auth_info.clone()
}

#[allow(unused)]
pub fn parse_group_extract_args(tokens: proc_macro2::TokenStream) -> HashMap<String, String> {
    let mut args_map = HashMap::<String, String>::new();
    for token in tokens.into_iter() {
        match token {
            // 遍历TokenTree::Group下的TokenStream
            proc_macro2::TokenTree::Group(ref group) => {
                let mut key = String::new();
                let mut value = String::new();
                let mut punctuation = String::new();
                let mut punctuation_counter = 0;

                // 获取组内的TokenStream并再次遍历
                let inner_tokens = group.stream();
                //println!("{}:{} {:?}", file!(), line!(), inner_tokens);
                for inner_tt in inner_tokens {
                    match inner_tt {
                        // ref 模式 https://rustwiki.org/zh-CN/rust-by-example/scope/borrow/ref.html
                        proc_macro2::TokenTree::Ident(ref ident) => {
                            if punctuation.is_empty() {
                                value = ident.clone().to_string();
                            } else {
                                if punctuation_counter >= 1 {
                                    key = key + &*ident.clone().to_string();
                                }
                            }
                        }
                        proc_macro2::TokenTree::Punct(ref punct) => {
                            if punct.to_string() == ":" {
                                punctuation_counter += 1;
                                punctuation = punct.clone().to_string();
                                if punctuation_counter > 1 {
                                    key = key + &*punct.clone().to_string();
                                }
                            } else if punct.to_string() == "," {
                                args_map.insert(key.clone(), value.clone());
                                key = String::new();
                                value = String::new();
                                punctuation = String::new();
                                punctuation_counter = 0;
                            } else {
                                key = key + &*punct.clone().to_string();
                            }
                        }
                        // 可以根据需要处理更多类型...
                        _ => (), // 处理其他类型或忽略
                    }
                }
                if !key.is_empty() && !value.is_empty() {
                    args_map.insert(key.clone(), value.clone());
                }
            }
            // 处理其他类型的TokenTree...
            _ => (), // 或者忽略非Group类型的TokenTree
        }
    }
    //println!("{}:{} {:?}", file!(), line!(), args_map);
    args_map
}

#[allow(unused)]
pub fn parse_group_extract_scope(tokens: proc_macro2::TokenStream) -> HashMap<String, String> {
    let mut args_map = HashMap::<String, String>::new();
    for token in tokens.into_iter() {
        match token {
            // 遍历TokenTree::Group下的TokenStream
            proc_macro2::TokenTree::Group(ref group) => {
                let mut key = String::new();
                let mut value = String::new();
                let mut punctuation = String::new();
                let mut punctuation_counter = 0;

                // 获取组内的TokenStream并再次遍历
                let inner_tokens = group.stream();
                for inner_tt in inner_tokens {
                    match inner_tt {
                        // ref 模式 https://rustwiki.org/zh-CN/rust-by-example/scope/borrow/ref.html
                        proc_macro2::TokenTree::Ident(ref ident) => {
                            if punctuation.is_empty() {
                                value = ident.clone().to_string();
                            } else {
                                if punctuation_counter >= 1 {
                                    key = key + &*ident.clone().to_string();
                                }
                            }
                        }
                        proc_macro2::TokenTree::Punct(ref punct) => {
                            if punct.to_string() == ":" {
                                punctuation_counter += 1;
                                punctuation = punct.clone().to_string();
                                if punctuation_counter > 1 {
                                    key = key + &*punct.clone().to_string();
                                }
                            } else if punct.to_string() == "," {
                                args_map.insert(key.clone(), value.clone());
                                key = String::new();
                                value = String::new();
                                punctuation = String::new();
                                punctuation_counter = 0;
                            } else {
                                key = key + &*punct.clone().to_string();
                            }
                        }
                        // 可以根据需要处理更多类型...
                        _ => (), // 处理其他类型或忽略
                    }
                }
                if !key.is_empty() && !value.is_empty() {
                    args_map.insert(key.clone(), value.clone());
                }
            }
            // 处理其他类型的TokenTree...
            _ => (), // 或者忽略非Group类型的TokenTree
        }
    }
    args_map
}
