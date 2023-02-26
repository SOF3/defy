# defy

[![GitHub actions](https://github.com/SOF3/defy/workflows/CI/badge.svg)](https://github.com/SOF3/defy/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/defy.svg)](https://crates.io/crates/defy)
[![crates.io](https://img.shields.io/crates/d/defy.svg)](https://crates.io/crates/defy)
[![docs.rs](https://docs.rs/defy/badge.svg)](https://docs.rs/defy)
[![GitHub](https://img.shields.io/github/last-commit/SOF3/defy)](https://github.com/SOF3/defy)
[![GitHub](https://img.shields.io/github/stars/SOF3/defy?style=social)](https://github.com/SOF3/defy)

Replacement for the [`yew::html!` macro](https://docs.rs/yew/latest/yew/macro.html.html)
with more Rust-idiomatic, editor-friendly syntax.

The syntax used in this crate is largely inspired by
[kotlinx.html](https://github.com/Kotlin/kotlinx.html) and
[horrorshow](https://docs.rs/horrorshow).

## Example

```rust
use defy::defy;

struct Datum {
    field:   &'static str,
    display: bool,
    label:   Label,
}
enum Label {
    First(i32),
    Second(u64),
}
let data = vec![
    Datum { field: "foo", display: false, label: Label::First(1) },
    Datum { field: "bar", display: true, label: Label::Second(2) },
];

let vnode = defy! {
    h1 {
        + "Hello world";
    }
    ul {
        for datum in data {
            let field = datum.field;
            if datum.display {
                li(data-length = field.len().to_string()) {
                    + field;
                }
            }
            match datum.label {
                Label::First(i) if i > 3 => {
                    h2 { +i; }
                }
                Label::Second(i) => {
                    h3 { +i; }
                }
                _ => { +"unmatched"; }
            }
        }
    }
};

// Rendering code omitted
assert_eq!(
    vnode_html.as_str().replace(['\n', ' '], ""),
    r#"
        <h1>Hello world</h1>
        <ul>
            unmatched
            <li data-length="3">bar</li>
            <h3>2</h3>
        </ul>
    "#
    .replace(['\n', ' '], "")
);
```

## Why invent another syntax?

Yew already provides several editor plugins
to help make editors treat `html!` blocks as HTML syntax.
However the editor covergae is not complete,
and interacts weirdly with normal Rust syntax.
`defy` uses a syntax that resembles the normal Rust syntax
(a similar idea as [ron](https://github.com/ron-rs/ron))
and provides better syntactic sugar for constructs like for loops.
