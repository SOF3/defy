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

#[test]
fn test_keyword() {
    let _: ast::Input = syn::parse2(quote! {
        input(type = "checkbox");
    }).unwrap();
}
