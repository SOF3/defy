use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Result;

pub struct Input {
    pub configs: Vec<Config>,
    pub nodes:   Nodes,
}
impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut configs = Vec::new();
        while input.peek(syn::Token![@]) {
            configs.push(input.parse()?);
        }

        Ok(Self { configs, nodes: input.parse()? })
    }
}

mod config_kw {
    syn::custom_keyword!(__debug_print);
    syn::custom_keyword!(macro_path);
}
pub enum Config {
    DebugPrint { at: syn::Token![@], kw: config_kw::__debug_print },
    MacroPath { at: syn::Token![@], kw: config_kw::macro_path, path: syn::Path },
}
impl Parse for Config {
    fn parse(input: ParseStream) -> Result<Self> {
        let at = input.parse()?;
        let lh = input.lookahead1();
        Ok(if lh.peek(config_kw::__debug_print) {
            Config::DebugPrint { at, kw: input.parse()? }
        } else if lh.peek(config_kw::macro_path) {
            Config::MacroPath { at, kw: input.parse()?, path: input.parse()? }
        } else {
            return Err(lh.error());
        })
    }
}

pub struct Nodes {
    pub stmts: Vec<Stmt>,
}
impl Parse for Nodes {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut stmts = Vec::new();
        while !input.is_empty() {
            stmts.push(input.parse()?);
        }
        Ok(Self { stmts })
    }
}

pub enum Stmt {
    If(If),
    Match(Match),
    For(For),
    Let(Let),
    Text(Text),
    Node(Node),
}
impl Parse for Stmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let lh = input.lookahead1();

        Ok(if lh.peek(syn::Token![if]) {
            Stmt::If(input.parse()?)
        } else if lh.peek(syn::Token![match]) {
            Stmt::Match(input.parse()?)
        } else if lh.peek(syn::Token![for]) {
            Stmt::For(input.parse()?)
        } else if lh.peek(syn::Token![let]) {
            Stmt::Let(input.parse()?)
        } else if lh.peek(syn::Token![+]) {
            Stmt::Text(input.parse()?)
        } else if lh.peek(syn::Ident) {
            Stmt::Node(input.parse()?)
        } else {
            return Err(lh.error());
        })
    }
}

pub struct If {
    pub if_:    syn::Token![if],
    pub expr:   Box<syn::Expr>,
    pub braces: syn::token::Brace,
    pub body:   Nodes,
    pub else_:  Option<Else>,
}
impl Parse for If {
    fn parse(input: ParseStream) -> Result<Self> {
        let inner;
        Ok(Self {
            if_:    input.parse()?,
            expr:   Box::new(input.call(syn::Expr::parse_without_eager_brace)?),
            braces: syn::braced!(inner in input),
            body:   inner.parse()?,
            else_:  if input.peek(syn::Token![else]) { Some(input.parse()?) } else { None },
        })
    }
}
pub struct Else {
    pub else_:  syn::Token![else],
    pub braces: syn::token::Brace,
    pub body:   Nodes,
}
impl Parse for Else {
    fn parse(input: ParseStream) -> Result<Self> {
        let inner;
        Ok(Self {
            else_:  input.parse()?,
            braces: syn::braced!(inner in input),
            body:   inner.parse()?,
        })
    }
}

pub struct Match {
    pub match_: syn::Token![match],
    pub expr:   Box<syn::Expr>,
    pub braces: syn::token::Brace,
    pub arms:   Vec<Arm>,
}
impl Parse for Match {
    fn parse(input: ParseStream) -> Result<Self> {
        let inner;
        Ok(Self {
            match_: input.parse()?,
            expr:   Box::new(input.call(syn::Expr::parse_without_eager_brace)?),
            braces: syn::braced!(inner in input),
            arms:   {
                let mut arms = Vec::new();
                while !inner.is_empty() {
                    arms.push(inner.parse()?);
                }
                arms
            },
        })
    }
}

pub struct Arm {
    pub pat:       syn::Pat,
    pub guard:     Option<(syn::Token![if], Box<syn::Expr>)>,
    pub fat_arrow: syn::Token![=>],
    pub braces:    syn::token::Brace,
    pub body:      Nodes,
}
impl Parse for Arm {
    fn parse(input: ParseStream) -> Result<Self> {
        let inner;
        Ok(Self {
            pat:       syn::Pat::parse_multi_with_leading_vert(input)?,
            guard:     if input.peek(syn::Token![if]) {
                Some((input.parse()?, input.parse()?))
            } else {
                None
            },
            fat_arrow: input.parse()?,
            braces:    syn::braced!(inner in input),
            body:      inner.parse()?,
        })
    }
}

pub struct For {
    pub for_:   syn::Token![for],
    pub pat:    Box<syn::Pat>,
    pub in_:    syn::Token![in],
    pub iter:   Box<syn::Expr>,
    pub braces: syn::token::Brace,
    pub body:   Nodes,
}
impl Parse for For {
    fn parse(input: ParseStream) -> Result<Self> {
        let inner;
        Ok(Self {
            for_:   input.parse()?,
            pat:    Box::new(syn::Pat::parse_multi_with_leading_vert(input)?),
            in_:    input.parse()?,
            iter:   Box::new(input.call(syn::Expr::parse_without_eager_brace)?),
            braces: syn::braced!(inner in input),
            body:   inner.parse()?,
        })
    }
}

pub struct Let {
    pub let_: syn::Token![let],
    pub pat:  Box<syn::Pat>,
    pub eq:   syn::Token![=],
    pub expr: Box<syn::Expr>,
    pub semi: syn::Token![;],
}
impl Parse for Let {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            let_: input.parse()?,
            pat:  Box::new(syn::Pat::parse_multi_with_leading_vert(input)?),
            eq:   input.parse()?,
            expr: input.parse()?,
            semi: input.parse()?,
        })
    }
}

pub struct Text {
    pub add:  syn::Token![+],
    pub expr: Box<syn::Expr>,
    pub semi: syn::Token![;],
}
impl Parse for Text {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self { add: input.parse()?, expr: input.parse()?, semi: input.parse()? })
    }
}

pub struct Node {
    pub element: syn::Path,
    pub args:    NodeArgs,
    pub body:    NodeBody,
}
impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            element: parse_path_without_paren(input)?,
            args:    input.parse()?,
            body:    input.parse()?,
        })
    }
}

pub enum NodeBody {
    Semi(syn::Token![;]),
    Braced { braces: syn::token::Brace, children: Nodes },
}
impl Parse for NodeBody {
    fn parse(input: ParseStream) -> Result<Self> {
        let lh = input.lookahead1();

        Ok(if lh.peek(syn::Token![;]) {
            Self::Semi(input.parse()?)
        } else if lh.peek(syn::token::Brace) {
            let inner;
            Self::Braced { braces: syn::braced!(inner in input), children: inner.parse()? }
        } else {
            return Err(lh.error());
        })
    }
}

// Fn/FnMut/FnOnce cannot be a component,
// so it is safe to steal the parentheses syntax for node arguments.
fn parse_path_without_paren(input: ParseStream) -> Result<syn::Path> {
    let leading_colon =
        if input.peek(syn::Token![::]) { Some(input.parse::<syn::Token![::]>()?) } else { None };

    let mut segments = Punctuated::new();
    loop {
        let ident = input.parse()?;
        let arguments = if input.peek(syn::Token![<]) {
            syn::PathArguments::AngleBracketed(input.parse()?)
        } else {
            syn::PathArguments::None
        };
        segments.push_value(syn::PathSegment { ident, arguments });

        if input.peek(syn::Token![::]) {
            segments.push_punct(input.parse()?);
        } else {
            break;
        }
    }

    Ok(syn::Path { leading_colon, segments })
}

pub enum NodeArgs {
    None,
    Named { paren: syn::token::Paren, args: Punctuated<NodeArg, syn::Token![,]> },
    Rest { eq: syn::Token![=], arg: Box<syn::Expr> },
}
impl Parse for NodeArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let lh = input.lookahead1();
        Ok(if lh.peek(syn::Token![=]) {
            NodeArgs::Rest { eq: input.parse()?, arg: input.parse()? }
        } else if lh.peek(syn::token::Paren) {
            let inner;
            NodeArgs::Named {
                paren: syn::parenthesized!(inner in input),
                args:  Punctuated::parse_terminated(&inner)?,
            }
        } else if lh.peek(syn::token::Brace) || lh.peek(syn::Token![;]) {
            NodeArgs::None
        } else {
            return Err(lh.error());
        })
    }
}

pub struct NodeArg {
    pub ident: Punctuated<syn::Ident, syn::Token![-]>,
    pub value: Option<(syn::Token![=], Box<syn::Expr>)>,
}
impl Parse for NodeArg {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            ident: Punctuated::parse_separated_nonempty_with(input, syn::Ident::parse_any)?,
            value: if input.peek(syn::Token![=]) {
                Some((input.parse()?, input.parse()?))
            } else {
                None
            },
        })
    }
}
