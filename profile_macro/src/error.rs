macro_rules! error {
    ($span:expr, $message:expr) => {{
        let msg: &'static str = $message;
        
        return syn::Error::new($span, msg).to_compile_error().into();
    }};
}