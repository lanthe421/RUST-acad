use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::ParseStream, parse::Parse, punctuated::Punctuated, Expr, Token};

struct BTreeMapInput {
    pairs: Punctuated<KeyValue, Token![,]>,
}

struct KeyValue {
    key: Expr,
    value: Expr,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let key: Expr = content.parse()?;
        content.parse::<Token![,]>()?;
        let value: Expr = content.parse()?;
        Ok(KeyValue { key, value })
    }
}

impl Parse for BTreeMapInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pairs = Punctuated::parse_terminated(input)?;
        Ok(BTreeMapInput { pairs })
    }
}

#[proc_macro]
pub fn btreemap(input: TokenStream) -> TokenStream {
    let BTreeMapInput {pairs} = syn::parse_macro_input!(input as BTreeMapInput);

    let keys = pairs.iter().map(|kv| &kv.key);
    let values = pairs.iter().map(|kv| &kv.value);

    quote! {
        {
            let mut map = ::std::collections::BTreeMap::new();
            #( map.insert(#keys, #values); )*
            map
        }
    }
    .into()
}
