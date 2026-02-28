use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Expr, Lit, spanned::Spanned};

pub fn convert_expr_to_op(expr: &Expr) -> syn::Result<TokenStream2> {
    match expr {
        Expr::Call(call) => {
            let func = match &*call.func {
                Expr::Path(p) => p.path.segments.last().unwrap().ident.to_string(),
                _ => return Err(syn::Error::new(call.func.span(), "expected op name")),
            };

            match func.as_str() {
                "gt" => {
                    let v = parse_arg_f64(&call.args, 0)?;
                    Ok(quote! { ::bitspec_engine::predicate::PredOp::GtF64(#v) })
                }
                "lt" => {
                    let v = parse_arg_f64(&call.args, 0)?;
                    Ok(quote! { ::bitspec_engine::predicate::PredOp::LtF64(#v) })
                }
                "between" => {
                    let lo = parse_arg_f64(&call.args, 0)?;
                    let hi = parse_arg_f64(&call.args, 1)?;
                    Ok(quote! {
                        ::bitspec_engine::predicate::PredOp::BetweenF64 { lo: #lo, hi: #hi }
                    })
                }

                "starts_with" => {
                    let s = parse_arg_str(&call.args, 0)?;
                    Ok(quote! {
                        ::bitspec_engine::predicate::PredOp::StartsWith(#s)
                    })
                }
                "contains" => {
                    let s = parse_arg_str(&call.args, 0)?;
                    Ok(quote! {
                        ::bitspec_engine::predicate::PredOp::Contains(#s)
                    })
                }

                "eq" => {
                    let arg = &call.args[0];

                    if let Ok(b) = get_lit_bool(arg) {
                        return Ok(quote! {
                            ::bitspec_engine::predicate::PredOp::EqBool(#b)
                        });
                    }
                    if let Ok(s) = get_lit_str(arg) {
                        return Ok(quote! {
                            ::bitspec_engine::predicate::PredOp::EqStr(#s)
                        });
                    }

                    let f = expr_to_f64(arg)?;
                    Ok(quote! {
                        ::bitspec_engine::predicate::PredOp::EqF64(#f)
                    })
                }

                "mod_eq" => {
                    let m = parse_arg_u64(&call.args, 0)?;
                    let r = parse_arg_u64(&call.args, 1)?;
                    Ok(quote! {
                        ::bitspec_engine::predicate::PredOp::ModEq { m: #m, r: #r }
                    })
                }
                "mod_ne" => {
                    let m = parse_arg_u64(&call.args, 0)?;
                    let r = parse_arg_u64(&call.args, 1)?;
                    Ok(quote! {
                        ::bitspec_engine::predicate::PredOp::ModNe { m: #m, r: #r }
                    })
                }

                "all" => {
                    let inner = call
                        .args
                        .iter()
                        .map(convert_expr_to_op)
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(quote! {
                        {
                            static INNER: &[::bitspec_engine::predicate::PredOp] = &[
                                #( #inner ),*
                            ];
                            ::bitspec_engine::predicate::PredOp::All(INNER)
                        }
                    })
                }

                "any" => {
                    let inner = call
                        .args
                        .iter()
                        .map(convert_expr_to_op)
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(quote! {
                        {
                            static INNER: &[::bitspec_engine::predicate::PredOp] = &[
                                #( #inner ),*
                            ];
                            ::bitspec_engine::predicate::PredOp::Any(INNER)
                        }
                    })
                }

                "not" => {
                    let inner = convert_expr_to_op(&call.args[0])?;
                    Ok(quote! {
                        {
                            static INNER: ::bitspec_engine::predicate::PredOp = #inner;
                            ::bitspec_engine::predicate::PredOp::Not(&INNER)
                        }
                    })
                }

                other => Err(syn::Error::new(
                    call.func.span(),
                    format!("Unknown predicate op '{}'", other),
                )),
            }
        }

        _ => Err(syn::Error::new(
            expr.span(),
            "expected function-call predicate: gt(5), eq(\"x\"), all(...), etc.",
        )),
    }
}

fn parse_arg_f64(
    args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>,
    idx: usize,
) -> syn::Result<f64> {
    let arg = args
        .iter()
        .nth(idx)
        .ok_or_else(|| syn::Error::new(Span::call_site(), "Missing argument"))?;
    expr_to_f64(arg)
}

fn expr_to_f64(expr: &Expr) -> syn::Result<f64> {
    if let Expr::Lit(l) = expr {
        match &l.lit {
            Lit::Float(f) => return f.base10_parse(),
            Lit::Int(i) => return i.base10_parse::<i64>().map(|x| x as f64),
            _ => {}
        }
    }
    // Handle unary minus for negative numbers: -10
    if let Expr::Unary(u) = expr {
        if let syn::UnOp::Neg(_) = u.op {
            let val = expr_to_f64(&u.expr)?;
            return Ok(-val);
        }
    }

    Err(syn::Error::new(
        expr.span(),
        "Expected floating point literal",
    ))
}

fn parse_arg_u64(
    args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>,
    idx: usize,
) -> syn::Result<u64> {
    let arg = args
        .iter()
        .nth(idx)
        .ok_or_else(|| syn::Error::new(Span::call_site(), "Missing argument"))?;
    if let Expr::Lit(l) = arg {
        if let Lit::Int(i) = &l.lit {
            return i.base10_parse();
        }
    }
    Err(syn::Error::new(arg.span(), "Expected u64 literal"))
}

fn parse_arg_str(
    args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>,
    idx: usize,
) -> syn::Result<String> {
    let arg = args
        .iter()
        .nth(idx)
        .ok_or_else(|| syn::Error::new(Span::call_site(), "Missing argument"))?;
    get_lit_str(arg)
}

fn get_lit_str(expr: &Expr) -> syn::Result<String> {
    if let Expr::Lit(l) = expr {
        if let Lit::Str(s) = &l.lit {
            return Ok(s.value());
        }
    }
    Err(syn::Error::new(expr.span(), "Expected string literal"))
}

fn get_lit_bool(expr: &Expr) -> syn::Result<bool> {
    if let Expr::Lit(l) = expr {
        if let Lit::Bool(b) = &l.lit {
            return Ok(b.value);
        }
    }
    Err(syn::Error::new(expr.span(), "Expected bool literal"))
}
