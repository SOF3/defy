//! Replacement for the [yew `html` macro](https://docs.rs/yew/latest/yew/macro.html.html)
//! with more Rust-idiomatic, editor-friendly syntax.
//!
//! The syntax used in this crate is largely inspired by
//! [kotlinx.html](https://github.com/Kotlin/kotlinx.html) and
//! [horrorshow](https://docs.rs/horrorshow).
//!
//! # Example
//! ```
//! use defy::defy;
//!
//! # #[yew::function_component]
//! # fn Test() -> yew::Html {
//! struct Datum {
//!     field:   &'static str,
//!     display: bool,
//!     label:   Label,
//! }
//! enum Label {
//!     First(i32),
//!     Second(u64),
//! }
//! let data = vec![
//!     Datum { field: "foo", display: false, label: Label::First(1) },
//!     Datum { field: "bar", display: true, label: Label::Second(2) },
//! ];
//!
//! let vnode = defy! {
//!     h1 {
//!         + "Hello world";
//!     }
//!     ul {
//!         for datum in data {
//!             let field = datum.field;
//!             if datum.display {
//!                 li(data-length = field.len().to_string()) {
//!                     + field;
//!                 }
//!             }
//!             match datum.label {
//!                 Label::First(i) if i > 3 => {
//!                     h2 { +i; }
//!                 }
//!                 Label::Second(i) => {
//!                     h3 { +i; }
//!                     br;
//!                 }
//!                 _ => { +"unmatched"; }
//!             }
//!         }
//!     }
//! };
//! # return vnode
//! # }
//!
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() {
//! #     let vnode_html = yew::ServerRenderer::<Test>::new().render().await;
//! #     println!("{vnode_html}");
//! # /*
//! let vnode_html = /* omitted implementation rendering vnode into HTML string */;
//! # */
//! assert_eq!(
//!     canonicalize(vnode_html.as_str()),
//!     canonicalize(
//!         r#"
//! #         <!--<[rust_out::Test]>-->
//!         <h1>Hello world</h1>
//!         <ul>
//!             unmatched
//!             <li data-length="3">bar</li>
//!             <h3>2</h3>
//!             <br> <!-- we actually emitted <br/> here, but yew processed it into <br> -->
//!         </ul>
//! #         <!--</[rust_out::Test]>-->
//!         "#
//!     )
//! );
//! # }
//!
//! fn canonicalize(string: &str) -> String {
//!     // Omitted implementation: strips whitespaces and comments
//! #     let mut output = string.replace(['\n', ' '], "");
//! #     while let Some(pos) = output.find("<!--") {
//! #         if let Some(len) = output[pos..].find("-->") {
//! #             output = format!("{}{}", &output[..pos], &output[(pos+len+3)..]);
//! #         }
//! #     }
//! #     output
//! }
//! ```
//!
//! # Reference
//! ## HTML tag without children
//! ```
//! # /*
//! foo(a = b, c = d);
//! # */
//! ```
//! becomes
//! ```html
//! <foo a={b} c={d} />
//! ```
//!
//! ## HTML tag with children
//! ```
//! # /*
//! foo(a = b, c = d) { ... }
//! # */
//! ```
//! becomes
//! ```html
//! <foo a={b} c={d}> ... </foo>
//! ```
//!
//! # Text values
//! ```
//! # /*
//! + expr;
//! # */
//! ```
//! becomes
//! ```text
//! `{ expr }`
//! ```
//!
//! # Local variables
//! Local variables can be defined in the form of normal `let` statements.
//! However they must precede all non-`let` statements in a `{}` block
//! in order to preserve evaluation order.
//!
//! If executing a `let` statement after other contents is really necessary,
//! place them under a separate `if true {}` block.
//!
//! # If, If-else, For
//! Same as the normal Rust syntax, except the contents in braces are automatically `defy!`-ed.
//!
//! # Match
//! Same as the normal Rust syntax, except match arm bodies must be surrounded in braces,
//! and the contents inside are automatically `defy!`-ed.

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Error, Result};

mod ast;

/// See crate-level documentation.
#[proc_macro]
pub fn defy(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    run(ts.into()).unwrap_or_else(Error::into_compile_error).into()
}

struct Config {
    debug_print: bool,
    macro_path:  syn::Path,
}

fn run(ts: TokenStream) -> Result<TokenStream> {
    let input: ast::Input = syn::parse2(ts)?;

    let mut config =
        Config { debug_print: false, macro_path: syn::parse2(quote!(::yew::html)).unwrap() };
    for ast_config in input.configs {
        match ast_config {
            ast::Config::DebugPrint { at: _, kw: _ } => config.debug_print = true,
            ast::Config::MacroPath { at: _, kw: _, path } => config.macro_path = path,
        }
    }

    let output = emit(&config.macro_path, Span::call_site(), input.nodes)?;
    if config.debug_print {
        println!("{output}")
    }
    Ok(output)
}

fn emit(macro_path: &syn::Path, span: Span, nodes: ast::Nodes) -> Result<TokenStream> {
    let mut stmts = nodes.stmts.into_iter().peekable();

    let mut locals = Vec::new();
    while let Some(ast::Stmt::Let(..)) = stmts.peek() {
        let ast::Let { let_, pat, eq, expr, semi } = match stmts.next() {
            Some(ast::Stmt::Let(stmt)) => stmt,
            _ => unreachable!(),
        };
        locals.push(quote_spanned! { let_.span() =>
            #let_ #pat #eq #expr #semi
        });
    }

    let node_html: Vec<_> =
        stmts.map(|stmt| stmt_to_html(macro_path, stmt)).collect::<Result<_>>()?;
    Ok(quote_spanned! { span =>
        #(#locals)*
        #macro_path! {
            <>
                #(#node_html)*
            </>
        }
    })
}

fn stmt_to_html(macro_path: &syn::Path, stmt: ast::Stmt) -> Result<TokenStream> {
    Ok(match stmt {
        ast::Stmt::If(ast::If {
            if_,
            expr,
            braces: if_braces,
            body: if_body,
            else_: Some(ast::Else { else_, braces: else_braces, body: else_body }),
        }) => {
            let if_body = emit(macro_path, if_braces.span, if_body)?;
            let if_part = quote_spanned! { if_braces.span =>
                #if_ #expr { #if_body }
            };

            let else_body = emit(macro_path, else_braces.span, else_body)?;
            let else_part = quote_spanned! { else_braces.span =>
                #else_ { #else_body }
            };

            quote_spanned! { if_.span() =>
                { #if_part #else_part }
            }
        }
        ast::Stmt::If(ast::If { if_, expr, braces, body, else_: None }) => {
            let body = emit(macro_path, braces.span, body)?;
            quote_spanned! { if_.span() =>
                { #if_ #expr { #body } else { #macro_path! {} } }
            }
        }
        ast::Stmt::Match(ast::Match { match_, expr, braces, arms }) => {
            let arms: TokenStream = arms
                .into_iter()
                .map(|ast::Arm { pat, guard, fat_arrow, braces, body }| {
                    let guard = guard.map(|(if_, expr)| quote!(#if_ #expr));
                    let body = emit(macro_path, braces.span, body)?;
                    Ok(quote_spanned! { braces.span =>
                        #pat #guard #fat_arrow { #body }
                    })
                })
                .collect::<Result<_>>()?;

            quote_spanned! { braces.span =>
                { #match_ #expr {
                    #arms
                } }
            }
        }
        ast::Stmt::For(ast::For { for_, pat, iter, in_, braces, body }) => {
            let body = emit(macro_path, braces.span, body)?;
            quote_spanned! { in_.span() =>
                { #for_ ::std::iter::IntoIterator::into_iter(#iter).map(|#pat| { #body }) }
            }
        }
        ast::Stmt::Let(ast::Let { let_, .. }) => {
            return Err(Error::new_spanned(
                let_,
                "let statements must precede all other statements in a block",
            ))
        }
        ast::Stmt::Text(ast::Text { add, expr, semi: _ }) => {
            quote_spanned! { add.span =>
                { #expr }
            }
        }
        ast::Stmt::Node(ast::Node { element, args, body }) => {
            let args = args_to_html(args)?;
            match body {
                ast::NodeBody::Semi(semi) => quote_spanned! { semi.span =>
                    <#element #args />
                },
                ast::NodeBody::Braced { braces, children } => {
                    let children = emit(macro_path, braces.span, children)?;
                    quote_spanned! { braces.span =>
                        <#element #args>
                            { #children }
                        </#element>
                    }
                }
            }
        }
    })
}

fn args_to_html(args: ast::NodeArgs) -> Result<TokenStream> {
    Ok(match args {
        ast::NodeArgs::None => TokenStream::new(),
        ast::NodeArgs::Named { paren: _, args } => args
            .into_iter()
            .map(|ast::NodeArg { ident, value }| match value {
                None => quote_spanned! { ident.span() =>
                    {#ident}
                },
                Some((eq, value)) => quote_spanned! { eq.span =>
                    #ident = {#value}
                },
            })
            .collect(),
        ast::NodeArgs::Rest { eq, arg } => quote_spanned! { eq.span =>
            ..#arg
        },
    })
}
