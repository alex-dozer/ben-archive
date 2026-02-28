// #[macro_export]
// macro_rules! bail {
//     ($span:expr, $msg:expr) => {
//         return ::quote::quote_spanned!($span => compile_error!($msg);)
//     };
// }

// pub fn compile_error(span: Span, msg: &str) -> TokenStream2 {
//     let t = quote::quote_spanned!(span => compile_error!(#msg););
//     t.into()
// }

use proc_macro2::TokenStream as TokenStream2;

/// Immediately returns a syn::Error from the current function.
///
/// # Patterns
///
/// 1. **Schema-Aware**: Adds "BenSchema: ... (table X)" prefix.
///    `bail!(spec, span, "My Error")`
///
/// 2. **Generic**: Standard error.
///    `bail!(span, "My Error")`
///      
#[macro_export]
macro_rules! bail {

    ($spec:expr, $span:expr, $($arg:tt)*) => {
        return Err(syn::Error::new(
            $span,
            format!("BenSchema: {} (table {})", format!($($arg)*), $spec.table)
        ))
    };


    ($span:expr, $($arg:tt)*) => {
        return Err(syn::Error::new($span, format!($($arg)*)))
    };
}

pub fn compile_error(span: proc_macro2::Span, msg: &str) -> TokenStream2 {
    syn::Error::new(span, msg).to_compile_error()
}
