use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{braced, parenthesized, parse::Parse, parse_macro_input, punctuated::Punctuated, token::{Brace, Paren}, Ident, Token};

struct FuncStruct {
    pub_token: Option<Token![pub]>,
    fn_token: Token![fn],
    ident: Ident,
    _paren: Paren,
    params: Punctuated<TokenStream2, Token![,]>,
    arrow: Option<Token![->]>,
    return_type: Option<Ident>,
    _brace_token: Brace,
    body: TokenStream2
}

impl quote::ToTokens for FuncStruct {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let FuncStruct { pub_token, fn_token, ident, _paren, params, arrow, return_type, _brace_token, body } = self;
        *tokens = quote! {
            #pub_token #fn_token #ident (#params) #arrow #return_type {
                #body
            }
        };
    }
}

impl Parse for FuncStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let content2;
        let has_arrow;

        Ok(FuncStruct {
            pub_token: input.parse()?,
            fn_token: input.parse()?,
            ident: input.parse()?,
            _paren: parenthesized!(content in input),
            params: content.parse_terminated(TokenStream2::parse, Token![,])?,
            arrow: {
                let parse: Option<_> = input.parse()?;
                has_arrow = parse.is_some();

                parse
            },
            return_type: if has_arrow {
                Some(input.parse()?)
            } else {
                None
            },
            _brace_token: braced!(content2 in input),
            body: content2.call(TokenStream2::parse)?
        })
    }
}

#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as FuncStruct);
    let path = if attr.is_empty() {
        func.ident.to_string()
    } else {
        parse_macro_input!(attr as syn::Path).into_token_stream().to_string() + "::" + &func.ident.to_string()
    };

    let body  = std::mem::take(&mut func.body);
    func.body = quote! {
        #[cfg(debug_assertions)]
        let __905d5516_de9e_4cbe_818c_b43ae89fbf8c = simple_profiler::profiler::FunctionCall::new(#path);;
        #body
    };
    
    func.to_token_stream().into()
}