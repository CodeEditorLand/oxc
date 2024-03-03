use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, token::Eq, Expr, ExprLit, Item, ItemEnum, ItemStruct};

pub fn ser_raw(item: &mut Item) -> TokenStream {
    match item {
        Item::Struct(item) => modify_struct(item),
        Item::Enum(item) => modify_enum(item),
        _ => panic!("Only use `ser_raw` attribute on structs and enums"),
    }
}

fn modify_struct(item: &mut ItemStruct) -> TokenStream {
    quote! {
        #[derive(::layout_inspect::Inspect)]
        #item
    }
}

fn modify_enum(item: &mut ItemEnum) -> TokenStream {
    // Add explicit discriminant to all variants
    let len = item.variants.len();
    assert!(len <= 254, "Too many enum variants");
    for (index, variant) in item.variants.iter_mut().enumerate() {
        let discriminant = if index < len - 1 { u8::try_from(index).unwrap() } else { 254 };
        variant.discriminant = Some((
            Eq { spans: [variant.ident.span()] },
            Expr::Lit(ExprLit { attrs: vec![], lit: parse_quote!(#discriminant) }),
        ));
    }

    quote! {
        #[derive(::layout_inspect::Inspect)]
        #[repr(u8)]
        #item
    }
}
