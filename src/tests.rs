#![cfg(test)]

use quote::quote;

use crate::ast;

#[test]
fn test_match() {
    let _: ast::Input = syn::parse2(quote! {
        match route {
            Route::List => {
                pages::comp;
            }
        }
    }).unwrap();
}
