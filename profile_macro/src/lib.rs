#[macro_use]
mod error;

use derive_syn_parse::Parse;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{Block, Ident, ItemFn, LitStr, Path, Stmt, Token, parse::ParseStream, parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned, token::Brace};

mod keywords {
    use syn::custom_keyword;

    custom_keyword!(name);
}

#[derive(Parse)]
struct Name {
    _name_ident: keywords::name,
    _equals: Token![=],
    name: LitStr
}

#[derive(Parse)]
enum PathOrName {
    #[peek(keywords::name, name = "Name")]
    Name(Name),
    #[peek(Ident, name = "Path")]
    Path(Path),
}

impl PathOrName {
    pub fn span(&self) -> Span {
        match self {
            PathOrName::Path(path) => path.span(),
            PathOrName::Name(name) => name._name_ident.span().join(name.name.span()).unwrap_or(name._name_ident.span()),
        }
    }
}

#[derive(Parse)]
struct Attrs {
    #[call(|input: ParseStream| input.parse_terminated(PathOrName::parse, Token![,]))]
    attrs: Punctuated<PathOrName, Token![,]>
}

#[derive(Parse)]
enum FnOrBlock {
    #[peek(Brace, name = "Block")]
    Block(Block),
    #[peek_with(|_| true, name = "Function")]
    Function(ItemFn)
}

#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut code = parse_macro_input!(item as FnOrBlock);

    let macro_input = parse_macro_input!(attr as Attrs);

    let mut name = None;
    let mut path = None;
    for attr in macro_input.attrs {
        let span = attr.span();
        match attr {
            PathOrName::Path(p) => match path {
                Some(_) => error!(span, "Path already specified."),
                None => path = Some(p)
            },
            PathOrName::Name(n) => match name {
                Some(_) => error!(span, "Name already specified."),
                None => name = Some(n)
            }
        }
    }

    let name = name.map(|name| name.name.value())
        .or(match &code {
            FnOrBlock::Block(_) => None,
            FnOrBlock::Function(item_fn) => Some(item_fn.sig.ident.to_string()),
        });
    
    let name = name.expect("Must provide a name for blocks.");

    let path = match path {
        Some(path) => path.into_token_stream().to_string() + "::" + &name,
        None => name
    };

    let block = match &mut code {
        FnOrBlock::Block(block) => block,
        FnOrBlock::Function(item_fn) => &mut item_fn.block,
    };

    let new_stmt: Stmt = parse_quote! {
        #[cfg(debug_assertions)]
        let __905d5516_de9e_4cbe_818c_b43ae89fbf8c = simple_profiler::profiler::FunctionCall::new(#path);
    };
    block.stmts.insert(0, new_stmt);
    
    match code {
        FnOrBlock::Block(block) => block.to_token_stream().into(),
        FnOrBlock::Function(item_fn) => item_fn.to_token_stream().into(),
    }
}