use {
    proc_macro::TokenStream,
    proc_macro2::{Delimiter, Group, Ident, Span},
    quote::ToTokens,
    syn::{
        punctuated::Punctuated,
        token::{Bracket, Eq, Pound},
        AttrStyle,
        Attribute,
        Expr,
        ExprLit,
        ImplItem,
        Item,
        Lit,
        LitStr,
        Meta,
        MetaNameValue,
        Path,
        PathArguments,
        PathSegment,
        TraitItem,
    },
};

// TODO:
// - Inline KaTeX instead of using jsdelivr.
// - Add support for images.
// - Return proper errors instead of panicking (probably).

#[allow(dead_code)]
mod katex;

#[proc_macro_attribute]
pub fn docext(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        panic!("#[docext] attribute does not take any arguments");
    }

    // Try interpreting the input as a module item.
    match syn::parse::<Item>(item).unwrap() {
        Item::Const(mut c) => {
            add_tex(&mut c.attrs);
            c.to_token_stream().into()
        }
        Item::Enum(mut e) => {
            add_tex(&mut e.attrs);
            e.to_token_stream().into()
        }
        Item::ExternCrate(mut c) => {
            add_tex(&mut c.attrs);
            c.to_token_stream().into()
        }
        Item::Fn(mut f) => {
            add_tex(&mut f.attrs);
            f.to_token_stream().into()
        }
        Item::ForeignMod(mut m) => {
            add_tex(&mut m.attrs);
            m.to_token_stream().into()
        }
        Item::Impl(mut i) => {
            add_tex(&mut i.attrs);
            i.to_token_stream().into()
        }
        Item::Macro(mut m) => {
            add_tex(&mut m.attrs);
            m.to_token_stream().into()
        }
        Item::Mod(mut m) => {
            add_tex(&mut m.attrs);
            m.to_token_stream().into()
        }
        Item::Static(mut s) => {
            add_tex(&mut s.attrs);
            s.to_token_stream().into()
        }
        Item::Struct(mut s) => {
            add_tex(&mut s.attrs);
            s.to_token_stream().into()
        }
        Item::Trait(mut t) => {
            add_tex(&mut t.attrs);
            t.to_token_stream().into()
        }
        Item::TraitAlias(mut t) => {
            add_tex(&mut t.attrs);
            t.to_token_stream().into()
        }
        Item::Type(mut t) => {
            add_tex(&mut t.attrs);
            t.to_token_stream().into()
        }
        Item::Union(mut u) => {
            add_tex(&mut u.attrs);
            u.to_token_stream().into()
        }
        Item::Use(mut u) => {
            add_tex(&mut u.attrs);
            u.to_token_stream().into()
        }
        Item::Verbatim(v) => {
            // Try interpreting the input as a trait item.
            match syn::parse::<TraitItem>(v.into()).unwrap() {
                TraitItem::Const(mut c) => {
                    add_tex(&mut c.attrs);
                    c.to_token_stream().into()
                }
                TraitItem::Fn(mut f) => {
                    add_tex(&mut f.attrs);
                    f.to_token_stream().into()
                }
                TraitItem::Type(mut t) => {
                    add_tex(&mut t.attrs);
                    t.to_token_stream().into()
                }
                TraitItem::Macro(mut m) => {
                    add_tex(&mut m.attrs);
                    m.to_token_stream().into()
                }
                TraitItem::Verbatim(v) => {
                    // Try interpreting the input as an impl item.
                    match syn::parse::<ImplItem>(v.into()).unwrap() {
                        ImplItem::Const(mut c) => {
                            add_tex(&mut c.attrs);
                            c.to_token_stream().into()
                        }
                        ImplItem::Fn(mut f) => {
                            add_tex(&mut f.attrs);
                            f.to_token_stream().into()
                        }
                        ImplItem::Type(mut t) => {
                            add_tex(&mut t.attrs);
                            t.to_token_stream().into()
                        }
                        ImplItem::Macro(mut m) => {
                            add_tex(&mut m.attrs);
                            m.to_token_stream().into()
                        }
                        other => panic!("Unsupported impl item type {other:#?}"),
                    }
                }
                other => panic!("Unsupported trait item type {other:#?}"),
            }
        }
        other => panic!("Unsupported item type {other:#?}"),
    }
}

/// Add KaTeX to the doc comment of the given item.
fn add_tex(attrs: &mut Vec<Attribute>) {
    // Error if the item doesn't have a doc comment, since #[docext] wouldn't do
    // anything useful in this case.
    if !attrs.iter().any(|attr| {
        let Ok(name_value) = attr.meta.require_name_value() else {
            return false;
        };
        name_value.path.is_ident("doc") && name_value.path.segments.len() == 1
    }) {
        panic!("#[docext] only applies to items with doc comments");
    }

    let mut doc = String::new();

    // Remove doc comments from the attrs and collect them into a single string.
    *attrs = std::mem::take(attrs)
        .into_iter()
        .filter_map(|attr| {
            let Ok(name_value) = attr.meta.require_name_value() else {
                return Some(attr);
            };
            if !name_value.path.is_ident("doc") || name_value.path.segments.len() != 1 {
                return Some(attr);
            }

            let Expr::Lit(ExprLit {
                lit: Lit::Str(lit), ..
            }) = &name_value.value
            else {
                return Some(attr);
            };

            doc.push_str(&lit.value());
            doc.push('\n');
            None
        })
        .collect();

    // Collapse all multi-line math blocks into single lines to avoid rendering
    // issues.
    let doc = collapse_math(&doc);

    // Add the doc comment back to the attrs.
    attrs.push(Attribute {
        pound_token: Pound {
            spans: [Span::call_site()],
        },
        style: AttrStyle::Outer,
        bracket_token: Bracket {
            span: Group::new(Delimiter::Bracket, proc_macro2::TokenStream::new()).delim_span(),
        },
        meta: Meta::NameValue(MetaNameValue {
            path: Path {
                leading_colon: None,
                segments: Punctuated::from_iter([PathSegment {
                    ident: Ident::new("doc", Span::call_site()),
                    arguments: PathArguments::None,
                }]),
            },
            eq_token: Eq {
                spans: [Span::call_site()],
            },
            value: Expr::Lit(ExprLit {
                attrs: Default::default(),
                lit: Lit::Str(LitStr::new(&doc, Span::call_site())),
            }),
        }),
    });

    // Add the KaTeX CSS and JS to the doc comment, enabling TeX rending. The script
    // which does the actual rendering calls `renderMathInElement` on its
    // parent, so that the TeX is only loaded in the doc comment, not the entire
    // page.
    attrs.push(Attribute {
        pound_token: Pound::default(),
        style: AttrStyle::Outer,
        bracket_token: Bracket::default(),
        meta: Meta::NameValue(MetaNameValue {
            path: Path {
                leading_colon: None,
                segments: Punctuated::from_iter(
                    [PathSegment {
                        ident: Ident::new("doc", Span::call_site()),
                        arguments: PathArguments::None,
                    }]
                ),
            },
            eq_token: Eq::default(),
            value: Expr::Lit(ExprLit {
                attrs: Default::default(),
                lit: Lit::Str(LitStr::new(
                    &format!(r#"<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/katex.min.css" integrity="sha384-GvrOXuhMATgEsSwCs4smul74iXGOixntILdUW9XmUC6+HX0sLNAK3q71HotJqlAn" crossorigin="anonymous">
                        <script src="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/katex.min.js" integrity="sha384-cpW21h6RZv/phavutF+AuVYrr+dA8xD9zs6FwLpaCct6O9ctzYFfFr4dgmgccOTx" crossorigin="anonymous"></script>
                        <script src="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/contrib/auto-render.min.js" integrity="sha384-+VBxd3r6XgURycqtZ117nYw44OOcIax56Z4dCRWbxyPt0Koah1uHoK0o4+/RRE05" crossorigin="anonymous"></script>
                        <script>
                            var currentScript = document.currentScript;
                            document.addEventListener("DOMContentLoaded", function() {{
                                renderMathInElement(currentScript.parentElement, {{
                                    delimiters: [
                                        {{ left: '$$', right: '$$', display: true }},
                                        {{ left: '$', right: '$', display: false }},
                                    ],
                                }});
                            }});
                        </script>"#),
                    Span::call_site(),
                )),
            }),
        }),
    });
}

/// Replace all newlines in math mode with spaces. This avoids rendering issues.
/// For example, starting a line with "-" (minus) in math mode would cause the
/// markdown to render as a list and completely break the math.
///
/// This is implemented based on the [KaTeX auto-render script](https://github.com/KaTeX/KaTeX/blob/4f1d9166749ca4bd669381b84b45589f1500a476/contrib/auto-render/splitAtDelimiters.js).
fn collapse_math(mut text: &str) -> String {
    let mut result = String::new();
    result.reserve(text.len());
    loop {
        // Find the start of the math block.
        let Some(start) = text.find('$') else {
            // There are no more math blocks.
            result.push_str(text);
            break;
        };

        // The text before the math block does not need to change.
        result.push_str(&text[..start]);

        let delim = if text[start..].starts_with("$$") {
            "$$"
        } else {
            "$"
        };
        match find_math_end(text, delim, start) {
            Some(end) => {
                // Replace all newlines in the math block with spaces.
                result.push_str(&text[start..end].replace('\n', " "));
                text = &text[end..];
            }
            None => {
                // There is no closing delimiter, so there is no math block. The text does not
                // need to change.
                result.push_str(&text[start..]);
                break;
            }
        }
    }
    result
}

/// Find the end of the math block, while respecting braces. Return the byte
/// index pointing after the end of the closing delimiter of the math block.
fn find_math_end(text: &str, delim: &str, start: usize) -> Option<usize> {
    let start = start + delim.len();
    let mut chars = text[start..].char_indices().fuse().peekable();
    let mut depth = 0;
    while let Some((i, c)) = chars.next() {
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
        } else if c == '\\' {
            // Skip the next character, since it is escaped.
            chars.next();
        } else if c == '$' && depth <= 0 {
            // Might be the end of the math block.

            if delim == "$" {
                return Some(start + i + 1);
            }

            // The delim is "$$", so check the next character.
            if let Some((i, '$')) = chars.peek() {
                return Some(start + *i + 1);
            }
        }
    }
    None
}

#[cfg(test)]
mod test_collapse;
