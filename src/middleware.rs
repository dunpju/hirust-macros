use quote::quote;
use syn::{parse_macro_input, parse_str, ItemFn};

use auth;
use proc_macro::{TokenStream, TokenTree};

// 鉴权宏
pub(crate) fn middleware_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut is_tag = false;
    let mut tag = String::new();
    let mut is_desc = false;
    let mut desc = String::new();
    let mut is_handler = false;
    let mut handler = String::new();
    let mut is_auth = false;
    let mut auth = true;
    for arg in args.into_iter() {
        // TokenTree判断是Ident
        if matches!(&arg, TokenTree::Ident(_)) && "tag".eq(&arg.to_string()) {
            is_tag = true;
        }
        if is_tag && matches!(&arg, TokenTree::Literal(_)) {
            let temp = arg.to_string(); // rust 如何把代码块里的字符串拿到代码块外面来
            tag = temp;
            is_tag = false;
        }
        if matches!(&arg, TokenTree::Ident(_)) && "desc".eq(&arg.to_string()) {
            is_desc = true;
        }
        if is_desc && matches!(&arg, TokenTree::Literal(_)) {
            let temp = arg.to_string(); // rust 如何把代码块里的字符串拿到代码块外面来
            desc = temp.replace("\"", "");
            is_desc = false;
        }
        if matches!(&arg, TokenTree::Ident(_)) && "handler".eq(&arg.to_string()) {
            is_handler = true;
        }
        if is_handler && matches!(&arg, TokenTree::Literal(_)) {
            let temp = arg.to_string(); // rust 如何把代码块里的字符串拿到代码块外面来
            handler = temp;
            is_handler = false;
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
    }

    //println!("item {:?}", item);

    // 解析输入作为ItemFn类型，它是syn 提供的表示函数类型
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

    if tag.clone().eq("") {
        tag = format!("{:?}", sig.ident.clone().to_string());
    }

    let auth_info = auth::Auth {
        tag: tag.clone().replace("\"", ""),
        desc: desc.clone().replace("\"", ""),
        middlewares: handler.clone().replace("\"", ""),
        auth,
    };

    let _serialized = serde_json::to_string(&auth_info.clone()).unwrap();

    //utils::create_and_append("./resources/route", &serialized.as_str());
    //utils::create_and_append(dotenv!("ROUTE"), &serialized.as_str());

    let binding = handler.clone();
    let handler_temp = binding.replace("\"", "");

    let contents = format!(
        r#"{{
            let __auth_ok = {}({});
            if !__auth_ok.ok() {{
                return __auth_ok.response().respond_to(&req);
            }}
        }}"#,
        handler_temp,
        tag.clone().to_string()
    );

    // 将字符串解析为Rust代码片段
    let handler_expr = parse_str::<syn::Expr>(contents.as_str()).unwrap();

    // 抽取函数体语句
    let statements = block.stmts;

    // 使用解析输入重构函数，然后输出
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
    )
    .into()
}
