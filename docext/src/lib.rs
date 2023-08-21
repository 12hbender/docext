use {
    proc_macro::TokenStream,
    proc_macro2::{Ident, Span},
    quote::ToTokens,
    syn::{
        parse_macro_input,
        punctuated::Punctuated,
        token::{Bracket, Eq, Pound},
        AttrStyle,
        Attribute,
        Expr,
        ExprLit,
        ImplItem,
        Item,
        ItemMod,
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
// - Add support to also stick docext directly on a fn, impl, etc.
// - Add support for images.

#[allow(dead_code)]
mod katex;

#[proc_macro_attribute]
pub fn docext(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO Ensure that _attr is empty and give a nice error message otherwise
    let mut module = parse_macro_input!(item as ItemMod);
    recurse(&mut module);
    module.into_token_stream().into()
}

/// Recurse down the module tree, adding TeX support to doc comments.
fn recurse(module: &mut ItemMod) {
    add_tex(&mut module.attrs);
    let Some((_, items)) = module.content.as_mut() else {
        return;
    };

    for item in items.iter_mut() {
        match item {
            Item::Mod(m) => recurse(m),
            Item::Const(c) => add_tex(&mut c.attrs),
            Item::Enum(e) => add_tex(&mut e.attrs),
            Item::ExternCrate(_) => todo!(),
            Item::Fn(f) => add_tex(&mut f.attrs),
            Item::ForeignMod(m) => add_tex(&mut m.attrs),
            Item::Macro(m) => add_tex(&mut m.attrs),
            Item::Static(s) => add_tex(&mut s.attrs),
            Item::Struct(s) => add_tex(&mut s.attrs),
            Item::TraitAlias(t) => add_tex(&mut t.attrs),
            Item::Type(t) => add_tex(&mut t.attrs),
            Item::Union(u) => add_tex(&mut u.attrs),
            Item::Impl(i) => {
                add_tex(&mut i.attrs);
                for item in i.items.iter_mut() {
                    match item {
                        ImplItem::Const(c) => add_tex(&mut c.attrs),
                        ImplItem::Fn(f) => add_tex(&mut f.attrs),
                        ImplItem::Type(t) => add_tex(&mut t.attrs),
                        ImplItem::Macro(m) => add_tex(&mut m.attrs),
                        ImplItem::Verbatim(_) => continue,
                        _ => panic!("Unsupported impl item type"),
                    }
                }
            }
            Item::Trait(t) => {
                add_tex(&mut t.attrs);
                for item in t.items.iter_mut() {
                    match item {
                        TraitItem::Const(c) => add_tex(&mut c.attrs),
                        TraitItem::Fn(f) => add_tex(&mut f.attrs),
                        TraitItem::Type(t) => add_tex(&mut t.attrs),
                        TraitItem::Macro(m) => add_tex(&mut m.attrs),
                        TraitItem::Verbatim(_) => continue,
                        _ => panic!("Unsupported trait item type"),
                    }
                }
            }
            Item::Use(_) | Item::Verbatim(_) => continue,
            _ => panic!("Unsupported module item type"),
        }
    }
}

fn add_tex(attrs: &mut Vec<Attribute>) {
    if !attrs.iter_mut().any(|attr| {
        let Ok(name_value) = attr.meta.require_name_value() else {
            return false;
        };
        name_value.path.is_ident("doc") && name_value.path.segments.len() == 1
    }) {
        return;
    }

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
                    .into_iter(),
                ),
            },
            eq_token: Eq::default(),
            value: Expr::Lit(ExprLit {
                attrs: Default::default(),
                lit: Lit::Str(LitStr::new(
                    r#"<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/katex.min.css" integrity="sha384-GvrOXuhMATgEsSwCs4smul74iXGOixntILdUW9XmUC6+HX0sLNAK3q71HotJqlAn" crossorigin="anonymous">
                    <script src="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/katex.min.js" integrity="sha384-cpW21h6RZv/phavutF+AuVYrr+dA8xD9zs6FwLpaCct6O9ctzYFfFr4dgmgccOTx" crossorigin="anonymous"></script>
                    <script src="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/contrib/auto-render.min.js" integrity="sha384-+VBxd3r6XgURycqtZ117nYw44OOcIax56Z4dCRWbxyPt0Koah1uHoK0o4+/RRE05" crossorigin="anonymous"></script>
                    <script>
                        var docExtTexSupport;
                        if (!docExtTexSupport) {
                            docExtTexSupport = true;
                            document.addEventListener("DOMContentLoaded", function() {
                                renderMathInElement(document.body);
                            });
                        }
                    </script>"#,
                    Span::call_site(),
                )),
            }),
        }),
    });
}
