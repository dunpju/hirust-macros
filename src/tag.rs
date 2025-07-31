use proc_macro::{TokenStream, TokenTree};

pub(crate) fn tag_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut is_name = false;
    let mut _name = String::new();
    for arg in args.into_iter() {
        if matches!(&arg, TokenTree::Ident(_)) && "path".eq(&arg.to_string()) {
            is_name = true;
        }
        if is_name && matches!(&arg, TokenTree::Literal(_)) {
            let temp = arg.to_string();
            _name = temp.clone().replace("\"", "");
            is_name = false;
        }
    }
    input
}