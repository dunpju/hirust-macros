use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[allow(dead_code)]
pub(crate) fn head_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
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

    //let tokens = quote!(#sig);
    //let req_map = utils::parse_group_extract_args(tokens);

    //let (_path, _contents) = utils::parse_token(args, req_map);

    // 将字符串解析为Rust代码片段
    //let middleware_expr = parse_str::<syn::Expr>(contents.as_str()).unwrap();

    //let attr: Attribute = parse_quote! {
    //    #[actix_web::head(#path)]
    //};

    //attrs.push(attr);

    // 抽取函数体语句
    let statements = block.stmts;

    // 使用解析输入重构函数，然后输出
    quote!(
        // 在该函数上重复其他所有属性（保持不变）
        #(#attrs)*
        // 重构函数声明
        #vis #sig {
            //#middleware_expr;
            let __result = {
                #(#statements)*
            };
            // 返回结果
            return __result;
        }
    )
        .into()
}