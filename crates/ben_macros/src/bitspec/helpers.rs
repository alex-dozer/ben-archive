use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Expr, LitStr, spanned::Spanned};

use crate::bitspec::data_objs::{ParsedPredicate, ParsedThreshold};
use crate::bitspec::dsl;

pub struct ParsedBitspec {
    pub preds: Vec<ParsedPredicate>,
    pub thresh: Vec<ParsedThreshold>,
}

pub fn parse_struct_fields(input: &DeriveInput) -> ParsedBitspec {
    let mut preds = Vec::new();
    let mut thresh = Vec::new();

    let struct_fields = match &input.data {
        syn::Data::Struct(ds) => &ds.fields,
        _ => panic!("#[derive(Bitspec)] only supported on structs"),
    };

    for field in struct_fields {
        let field_ident = field.ident.as_ref().unwrap();
        let field_name = field_ident.to_string();

        for attr in &field.attrs {
            if !attr.path().is_ident("bspec") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("brule") {
                    preds.push(parse_brule(&field_name, meta)?);
                    return Ok(());
                }

                if meta.path.is_ident("thresholds") {
                    thresh.push(parse_thresholds(&field_name, meta)?);
                    return Ok(());
                }

                Err(meta.error("Unknown key inside #[bspec(...)]"))
            })
            .expect("failed parsing #[bspec]");
        }
    }

    ParsedBitspec { preds, thresh }
}

fn parse_brule(field: &str, meta: syn::meta::ParseNestedMeta) -> syn::Result<ParsedPredicate> {
    let mut rule_name: Option<String> = None;
    let mut op_expr: Option<Expr> = None;

    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("rule") {
            let lit: LitStr = nested.value()?.parse()?;
            rule_name = Some(lit.value());
            return Ok(());
        }

        if nested.path.is_ident("op") {
            let expr: Expr = nested.value()?.parse()?;
            op_expr = Some(expr);
            return Ok(());
        }

        Err(nested.error("unknown key in brule(...)"))
    })?;

    let rule_name = rule_name.ok_or_else(|| meta.error("brule requires rule=\"...\""))?;
    let op_raw = op_expr.ok_or_else(|| meta.error("brule requires op=..."))?;

    let op_tokens = dsl::convert_expr_to_op(&op_raw)?;

    Ok(ParsedPredicate {
        field_id: field.to_string(),
        rule_name,
        op_tokens,
    })
}

fn parse_thresholds(field: &str, meta: syn::meta::ParseNestedMeta) -> syn::Result<ParsedThreshold> {
    let mut rule = None;
    let mut fact = None;
    let mut op = None;
    let mut flags: Vec<(String, Vec<String>)> = Vec::new();
    let mut levels_raw: Option<String> = None;

    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("rule") {
            let lit: LitStr = nested.value()?.parse()?;
            rule = Some(lit.value());
            return Ok(());
        }

        if nested.path.is_ident("fact") {
            let lit: LitStr = nested.value()?.parse()?;
            fact = Some(lit.value());
            return Ok(());
        }

        if nested.path.is_ident("op") {
            let lit: LitStr = nested.value()?.parse()?;
            op = Some(lit.value());
            return Ok(());
        }

        if nested.path.is_ident("values") {
            let lit: LitStr = nested.value()?.parse()?;
            levels_raw = Some(lit.value());
            return Ok(());
        }

        if nested.path.is_ident("flags") {
            let lit: LitStr = nested.value()?.parse()?;
            let raw = lit.value();

            for clause in raw.split(',') {
                let clause = clause.trim();
                if clause.is_empty() {
                    continue;
                }

                let (flag, lvls) = clause
                    .split_once(':')
                    .ok_or_else(|| nested.error("flags entry must be FLAG:LEVEL1|LEVEL2"))?;

                let lvls_vec = lvls
                    .split(|c| c == '|' || c == ',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>();

                flags.push((flag.trim().to_string(), lvls_vec));
            }

            return Ok(());
        }

        Err(nested.error("unknown key in thresholds(...)"))
    })?;

    let raw = levels_raw.ok_or_else(|| meta.error("thresholds requires values=\"...\""))?;
    let mut levels = Vec::new();

    for entry in raw.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }

        let (name, val_str) = entry
            .split_once('=')
            .ok_or_else(|| meta.error("threshold entry must be NAME=VALUE"))?;

        let value = val_str
            .trim()
            .parse::<f64>()
            .map_err(|_| meta.error("threshold value must be f64"))?;

        levels.push((name.trim().to_string(), value));
    }

    Ok(ParsedThreshold {
        field_id: field.to_string(),
        rule_name: rule.ok_or_else(|| meta.error("thresholds requires rule=\"...\""))?,
        fact,
        flags,
        op,
        levels,
    })
}

pub fn make_codegen_items(
    parsed: &ParsedBitspec,
) -> syn::Result<(Vec<TokenStream2>, Vec<TokenStream2>, u16)> {
    let mut next_bit: u16 = 0;

    let mut pred_items = Vec::<TokenStream2>::new();
    for ParsedPredicate {
        field_id,
        rule_name: _,
        op_tokens,
    } in &parsed.preds
    {
        let bit = next_bit;
        next_bit += 1;

        pred_items.push(quote! {
            ::bitspec_engine::predicate::PredicateSpec {
                field_id: #field_id,
                bit: (1u64 << #bit),
                op: #op_tokens,
            }
        });
    }

    let mut thresh_items = Vec::<TokenStream2>::new();
    for ParsedThreshold {
        field_id,
        rule_name: _,
        fact,
        flags,
        op,
        levels,
    } in &parsed.thresh
    {
        let mut level_tokens = Vec::<TokenStream2>::new();

        for (lvl_name, lvl_val) in levels {
            let bit = next_bit;
            next_bit += 1;

            level_tokens.push(quote! {
                ::bitspec_engine::threshold::ThresholdLevel {
                    name: #lvl_name,
                    value: #lvl_val,
                    bit: #bit,
                }
            });
        }

        let op_ts = match op.as_deref() {
            Some("gt") => quote! { ::bitspec_engine::threshold::ThresholdOp::Gt },
            Some("gte") => quote! { ::bitspec_engine::threshold::ThresholdOp::Gte },
            Some("lt") => quote! { ::bitspec_engine::threshold::ThresholdOp::Lt },
            Some("lte") => quote! { ::bitspec_engine::threshold::ThresholdOp::Lte },
            Some("eq") => quote! { ::bitspec_engine::threshold::ThresholdOp::Eq },
            Some(other) => panic!("unknown threshold op '{}'", other),
            None => panic!("threshold requires op=..."),
        };

        let fact_ts = match fact {
            Some(s) => quote! { Some(#s) },
            None => quote! { None },
        };

        let flags_ts = {
            let mut outer = Vec::new();
            for (flag, lvl_list) in flags {
                let lvl_idents = lvl_list.iter();
                outer.push(quote! {
                    (#flag, &[ #( #lvl_idents ),* ])
                });
            }
            quote! { &[ #( #outer ),* ] }
        };

        thresh_items.push(quote! {
            ::bitspec_engine::threshold::ThresholdSpec {
                field_id: #field_id,
                fact_key: #fact_ts,
                flags: #flags_ts,
                threshold_op: #op_ts,
                levels: &[
                    #( #level_tokens ),*
                ],
            }
        });
    }

    Ok((pred_items, thresh_items, next_bit))
}
