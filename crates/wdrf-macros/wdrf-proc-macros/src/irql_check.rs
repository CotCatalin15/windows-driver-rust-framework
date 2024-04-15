use proc_macro2::TokenStream;

#[derive(deluxe::ParseMetaItem)]
struct IrqlCheckMetadataAttributes {
    irql: syn::Ident,

    #[deluxe(rest)]
    rest: std::collections::HashMap<syn::Path, syn::Expr>,
}

use quote::quote;

fn generate_default_irql_comare() -> syn::Expr {
    // Define the identifiers for the path segments
    let namespace = syn::Ident::new("wdrf_macros", proc_macro2::Span::call_site());
    let ident_irql_compare = syn::Ident::new("IrqlCompare", proc_macro2::Span::call_site());
    let ident_less_eq = syn::Ident::new("LessEq", proc_macro2::Span::call_site());

    // Create the path segments
    let namespace_segment_irql_compare = syn::PathSegment {
        ident: namespace,
        arguments: syn::PathArguments::None,
    };

    let path_segment_irql_compare = syn::PathSegment {
        ident: ident_irql_compare,
        arguments: syn::PathArguments::None,
    };
    let path_segment_less_eq = syn::PathSegment {
        ident: ident_less_eq,
        arguments: syn::PathArguments::None,
    };

    // Construct the full path
    let path = syn::Path {
        leading_colon: None,
        segments: vec![
            namespace_segment_irql_compare,
            path_segment_irql_compare,
            path_segment_less_eq,
        ]
        .into_iter()
        .collect(),
    };

    // Create the ExprPath
    let expr_path = syn::ExprPath {
        attrs: vec![],
        qself: None,
        path,
    };

    syn::Expr::Path(expr_path)
}

/// This is an incomplete implementation of all the features required
/// But it does the job for now
///  
pub(crate) fn irql_check_attr_impl(
    attr: TokenStream,
    item: TokenStream,
) -> deluxe::Result<TokenStream> {
    let IrqlCheckMetadataAttributes { irql, rest } =
        deluxe::parse2::<IrqlCheckMetadataAttributes>(attr.clone())?;

    let mut default_compare = generate_default_irql_comare();
    for (key, value) in rest {
        if let Some(last_segment) = key.segments.last() {
            if last_segment.ident == "compare" {
                default_compare = value;
                break;
            }
        }
    }

    let mut tokens: syn::ItemFn = syn::parse2(item.clone()).unwrap();

    let irql_tokens: TokenStream = quote! {
        fn _tmp_fnc() {
            wdrf_macros::irql_check_compare_and_panic::<{ #default_compare }>(#irql);
        }
    };

    let mut irql_tokens: syn::ItemFn = syn::parse2(irql_tokens).unwrap();

    irql_tokens.block.stmts.append(&mut tokens.block.stmts);
    tokens.block.stmts = irql_tokens.block.stmts;

    Ok(quote! {
        #tokens
    })
}
