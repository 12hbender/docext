use {
    base64::Engine,
    proc_macro::TokenStream,
    proc_macro2::{Ident, Span},
    quote::ToTokens,
    regex::Regex,
    std::{
        collections::HashSet,
        env,
        fs,
        path::{self, PathBuf},
    },
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
    url::Url,
};

mod parser;

// TODO:
// - Remove the dependency on url and base64 and implement this manually instead
// - Clean up the tests in example crate, make the explanations more clear.
//   (I.e. what is expected vs. what is not expected maybe.) Probably put it all
//   in one comment for easier checking. Or maybe put them each on separate
//   methods.
// - Add a feature to skip the image size check.

#[proc_macro_attribute]
pub fn docext(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        panic!("#[docext] attribute does not take any arguments");
    }

    // Try interpreting the input as a module item.
    match syn::parse::<Item>(item).unwrap() {
        Item::Const(mut c) => {
            update_doc(&mut c.attrs);
            c.to_token_stream().into()
        }
        Item::Enum(mut e) => {
            update_doc(&mut e.attrs);
            e.to_token_stream().into()
        }
        Item::ExternCrate(mut c) => {
            update_doc(&mut c.attrs);
            c.to_token_stream().into()
        }
        Item::Fn(mut f) => {
            update_doc(&mut f.attrs);
            f.to_token_stream().into()
        }
        Item::ForeignMod(mut m) => {
            update_doc(&mut m.attrs);
            m.to_token_stream().into()
        }
        Item::Impl(mut i) => {
            update_doc(&mut i.attrs);
            i.to_token_stream().into()
        }
        Item::Macro(mut m) => {
            update_doc(&mut m.attrs);
            m.to_token_stream().into()
        }
        Item::Mod(mut m) => {
            update_doc(&mut m.attrs);
            m.to_token_stream().into()
        }
        Item::Static(mut s) => {
            update_doc(&mut s.attrs);
            s.to_token_stream().into()
        }
        Item::Struct(mut s) => {
            update_doc(&mut s.attrs);
            s.to_token_stream().into()
        }
        Item::Trait(mut t) => {
            update_doc(&mut t.attrs);
            t.to_token_stream().into()
        }
        Item::TraitAlias(mut t) => {
            update_doc(&mut t.attrs);
            t.to_token_stream().into()
        }
        Item::Type(mut t) => {
            update_doc(&mut t.attrs);
            t.to_token_stream().into()
        }
        Item::Union(mut u) => {
            update_doc(&mut u.attrs);
            u.to_token_stream().into()
        }
        Item::Use(mut u) => {
            update_doc(&mut u.attrs);
            u.to_token_stream().into()
        }
        Item::Verbatim(v) => {
            // Try interpreting the input as a trait item.
            match syn::parse::<TraitItem>(v.into()).unwrap() {
                TraitItem::Const(mut c) => {
                    update_doc(&mut c.attrs);
                    c.to_token_stream().into()
                }
                TraitItem::Fn(mut f) => {
                    update_doc(&mut f.attrs);
                    f.to_token_stream().into()
                }
                TraitItem::Type(mut t) => {
                    update_doc(&mut t.attrs);
                    t.to_token_stream().into()
                }
                TraitItem::Macro(mut m) => {
                    update_doc(&mut m.attrs);
                    m.to_token_stream().into()
                }
                TraitItem::Verbatim(v) => {
                    // Try interpreting the input as an impl item.
                    match syn::parse::<ImplItem>(v.into()).unwrap() {
                        ImplItem::Const(mut c) => {
                            update_doc(&mut c.attrs);
                            c.to_token_stream().into()
                        }
                        ImplItem::Fn(mut f) => {
                            update_doc(&mut f.attrs);
                            f.to_token_stream().into()
                        }
                        ImplItem::Type(mut t) => {
                            update_doc(&mut t.attrs);
                            t.to_token_stream().into()
                        }
                        ImplItem::Macro(mut m) => {
                            update_doc(&mut m.attrs);
                            m.to_token_stream().into()
                        }
                        other => panic!("unsupported impl item type {other:#?}"),
                    }
                }
                other => panic!("unsupported trait item type {other:#?}"),
            }
        }
        other => panic!("unsupported item type {other:#?}"),
    }
}

/// Update the doc comments with KaTeX syntax rendering and image support.
fn update_doc(attrs: &mut Vec<Attribute>) {
    // Error if there is no doc comment, since #[docext] wouldn't do anything useful
    // in this case.
    if !attrs.iter().any(|attr| {
        let Ok(name_value) = attr.meta.require_name_value() else {
            return false;
        };
        name_value.path.is_ident("doc") && name_value.path.segments.len() == 1
    }) {
        panic!("#[docext] only applies to items with doc comments");
    }

    // Remove doc comments from the attrs and collect them into a single string.
    let mut doc = String::new();
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

    // Markdown options used by rustdoc.
    let opts = pulldown_cmark::Options::ENABLE_TABLES
        | pulldown_cmark::Options::ENABLE_FOOTNOTES
        | pulldown_cmark::Options::ENABLE_STRIKETHROUGH
        | pulldown_cmark::Options::ENABLE_TASKLISTS
        | pulldown_cmark::Options::ENABLE_SMART_PUNCTUATION;

    // Spans of code blocks in the doc comment. These are needed to ensure that math
    // is not rendered inside of markdown code blocks.
    let mut code_sections = Vec::new();
    // Collect code sections from doc comment.
    for (ev, range) in pulldown_cmark::Parser::new_ext(&doc, opts).into_offset_iter() {
        if let pulldown_cmark::Event::Code(..)
        | pulldown_cmark::Event::Start(pulldown_cmark::Tag::CodeBlock(..)) = ev
        {
            code_sections.push(range);
        }
    }

    // Regex matching ASCII punctuation characters (https://spec.commonmark.org/0.31.2/#ascii-punctuation-character).
    let punctuation = Regex::new(
        r##"(?<punct>[\!\"\#\$\%\&\'\(\)\*\+\,\-\.\/\:\;<\=>\?\@\[\\\]\^\_\`\{\|\}\~])"##,
    )
    .unwrap();

    // Math sections in the doc comment, used to avoid encoding images as base64
    // data if they happen to be inside of a math block.
    let mut math_sections = Vec::new();

    let mut doc: String = parser::parse_math(&doc)
        .into_iter()
        .map(|event| match event {
            parser::Event::Text(text) => {
                // Leave all regular markdown text unchanged.
                text.to_owned()
            }
            parser::Event::Math(math, range)
                if code_sections
                    .iter()
                    .any(|section| section.start <= range.start && range.end <= section.end) =>
            {
                // Math sections inside code blocks are not rendered by KaTeX. Don't escape
                // punctuation, leave them unchanged.
                math.to_owned()
            }
            parser::Event::Math(math, range) => {
                if math.lines().any(|line| line.trim().is_empty()) {
                    // The rustdoc markdown renderer interprets blank lines as starting a new
                    // paragraph, which would break the math.
                    panic!("blank lines in math blocks are not supported");
                }
                math_sections.push(range);
                // Escape all punctuation characters. This is to ensure that the markdown
                // renderer in rustdoc doesn't break the math. (Otherwise, for example starting
                // a line with "-" (minus) in the math block would cause the
                // markdown to render as a list and completely break the math,
                // or for example writing $[a](b)$ would render as a link.)
                punctuation.replace_all(math, r"\$punct").into_owned()
            }
        })
        .collect();

    // Paths to local images used in the doc comment.
    let mut imgs = HashSet::new();
    // Collect all images from the doc comment that are not URLs. These will be
    // encoded as base64 data and inserted into the doc comment as HTML tags, to
    // be loaded and rendered by an image rendering script.
    for (ev, range) in pulldown_cmark::Parser::new_ext(&doc, opts).into_offset_iter() {
        let pulldown_cmark::Event::Start(pulldown_cmark::Tag::Image {
            dest_url: path_or_url,
            ..
        }) = ev
        else {
            // Not an image tag.
            continue;
        };
        if Url::parse(&path_or_url).is_ok() {
            // This is a URL, so it is not a local image.
            continue;
        }
        if math_sections
            .iter()
            .any(|section| section.start <= range.start && range.end <= section.end)
        {
            // The image is inside a math block, so it should not be encoded as base64.
            // This avoids the following (extreme) edge case: if there is a math block such
            // as $![img](path/does/not_exist.png)$, docext would panic because it couldn't
            // find the image at that path. The root cause of the issue is that the markdown
            // parser does not recognize math blocks, so it interprets the math
            // as a regular image tag.
            continue;
        }
        imgs.insert(path_or_url.into_string());
    }

    // Add the KaTeX CSS and JS to the doc comment, enabling TeX rending. Add a
    // rendering script which calls `renderMathInElement` on its parent, so
    // that the TeX is only rendered in the doc comment, not the entire page.
    doc.push_str(r#"
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/katex.min.css" integrity="sha384-GvrOXuhMATgEsSwCs4smul74iXGOixntILdUW9XmUC6+HX0sLNAK3q71HotJqlAn" crossorigin="anonymous">
<script src="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/katex.min.js" integrity="sha384-cpW21h6RZv/phavutF+AuVYrr+dA8xD9zs6FwLpaCct6O9ctzYFfFr4dgmgccOTx" crossorigin="anonymous"></script>
<script src="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/contrib/auto-render.min.js" integrity="sha384-+VBxd3r6XgURycqtZ117nYw44OOcIax56Z4dCRWbxyPt0Koah1uHoK0o4+/RRE05" crossorigin="anonymous"></script>
<script>
(function() {
    var parent = document.currentScript.parentElement;
    document.addEventListener("DOMContentLoaded", function() {
        renderMathInElement(parent, {
            delimiters:[
                { left: "$$", right: "$$", display: true },
                { left: "$", right: "$", display: false }
            ]
        })
    });
})()
</script>"#);

    // Encode all images as base64 data inside of span attributes. Later, a script
    // will replace the src attributes of the images with the base64 data. This
    // is done to facilitate high-quality IDE hovers, since putting the base64 data
    // directly in the middle of the hover could result in bad UX.
    for img in imgs.iter() {
        // Load the image relative to the current crate.
        let mut path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        path.push(img);

        // Ensure that the file is not too large, otherwise the compiler might crash.
        let metadata = fs::metadata(&path).unwrap_or_else(|_| {
            panic!(
                r#"failed to stat image: "{}", is the file missing?"#,
                path.to_string_lossy()
            )
        });
        if metadata.len() > 1024 * 1024 {
            panic!(
                r#"image file too large: "{}", max size is 1MB"#,
                path.to_string_lossy()
            );
        }

        // Encode the image as base64.
        let data = fs::read(&path).unwrap_or_else(|_| {
            panic!(
                r#"failed to read image: "{}", is the file missing?"#,
                path.to_string_lossy()
            )
        });
        let base64 = base64::engine::general_purpose::STANDARD.encode(&data);

        // The data URL requires a MIME type.
        let mime = mime(&path);

        // Add a span containing the image data encoded as base64.
        doc.push('\n');
        doc.push_str(&format!(
            r#"<span class="docext-img" data-src="{img}" data-img="data:{mime};base64,{base64}"></span>"#,
        ));
    }

    if !imgs.is_empty() {
        // Add the image rendering script.
        doc.push_str(r#"
<script>
(function() {
    var elem = document.currentScript.parentElement;
    document.addEventListener("DOMContentLoaded", function() {
        elem.querySelectorAll(".docext-img").forEach(function(e) {
            elem.querySelectorAll("img[src='" + e.getAttribute("data-src") + "']").forEach(function(i) {
                i.src = e.getAttribute("data-img");
            });
        });
    });
})()
</script>"#);
    }

    // Create the modified doc attribute.
    attrs.push(Attribute {
        pound_token: Pound::default(),
        style: AttrStyle::Outer,
        bracket_token: Bracket::default(),
        meta: Meta::NameValue(MetaNameValue {
            path: Path {
                leading_colon: None,
                segments: Punctuated::from_iter([PathSegment {
                    ident: Ident::new("doc", Span::call_site()),
                    arguments: PathArguments::None,
                }]),
            },
            eq_token: Eq::default(),
            value: Expr::Lit(ExprLit {
                attrs: Default::default(),
                lit: Lit::Str(LitStr::new(&doc, Span::call_site())),
            }),
        }),
    });
}

/// Get the MIME type of the given image path based on its extension.
fn mime(path: &path::Path) -> &'static str {
    let ext = path.extension().unwrap_or_else(|| {
        panic!(
            r#"image path has no extension: "{}""#,
            path.to_string_lossy()
        )
    });
    match ext.to_string_lossy().as_ref() {
        "apng" => "image/apng",
        "avif" => "image/avif",
        "gif" => "image/gif",
        "jpg" | "jpeg" | "jfif" | "pjpeg" | "pjp" => "image/jpeg",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "ico" | "cur" => "image/x-icon",
        "tif" | "tiff" => "image/tiff",
        _ => panic!(r#"unsupported image format: "{}""#, ext.to_string_lossy()),
    }
}
