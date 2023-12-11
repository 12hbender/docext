use {
    base64::Engine,
    proc_macro::TokenStream,
    proc_macro2::{Ident, Span},
    quote::ToTokens,
    regex::Regex,
    std::env,
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
// - I could actually inline the KaTeX code inside some data-code attributes
// - Remove the dependency on url and base64

#[proc_macro_attribute]
pub fn docext(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        panic!("#[docext] attribute does not take any arguments");
    }

    // Try interpreting the input as a module item.
    match syn::parse::<Item>(item).unwrap() {
        Item::Const(mut c) => {
            add_docext(&mut c.attrs);
            c.to_token_stream().into()
        }
        Item::Enum(mut e) => {
            add_docext(&mut e.attrs);
            e.to_token_stream().into()
        }
        Item::ExternCrate(mut c) => {
            add_docext(&mut c.attrs);
            c.to_token_stream().into()
        }
        Item::Fn(mut f) => {
            add_docext(&mut f.attrs);
            f.to_token_stream().into()
        }
        Item::ForeignMod(mut m) => {
            add_docext(&mut m.attrs);
            m.to_token_stream().into()
        }
        Item::Impl(mut i) => {
            add_docext(&mut i.attrs);
            i.to_token_stream().into()
        }
        Item::Macro(mut m) => {
            add_docext(&mut m.attrs);
            m.to_token_stream().into()
        }
        Item::Mod(mut m) => {
            add_docext(&mut m.attrs);
            m.to_token_stream().into()
        }
        Item::Static(mut s) => {
            add_docext(&mut s.attrs);
            s.to_token_stream().into()
        }
        Item::Struct(mut s) => {
            add_docext(&mut s.attrs);
            s.to_token_stream().into()
        }
        Item::Trait(mut t) => {
            add_docext(&mut t.attrs);
            t.to_token_stream().into()
        }
        Item::TraitAlias(mut t) => {
            add_docext(&mut t.attrs);
            t.to_token_stream().into()
        }
        Item::Type(mut t) => {
            add_docext(&mut t.attrs);
            t.to_token_stream().into()
        }
        Item::Union(mut u) => {
            add_docext(&mut u.attrs);
            u.to_token_stream().into()
        }
        Item::Use(mut u) => {
            add_docext(&mut u.attrs);
            u.to_token_stream().into()
        }
        Item::Verbatim(v) => {
            // Try interpreting the input as a trait item.
            match syn::parse::<TraitItem>(v.into()).unwrap() {
                TraitItem::Const(mut c) => {
                    add_docext(&mut c.attrs);
                    c.to_token_stream().into()
                }
                TraitItem::Fn(mut f) => {
                    add_docext(&mut f.attrs);
                    f.to_token_stream().into()
                }
                TraitItem::Type(mut t) => {
                    add_docext(&mut t.attrs);
                    t.to_token_stream().into()
                }
                TraitItem::Macro(mut m) => {
                    add_docext(&mut m.attrs);
                    m.to_token_stream().into()
                }
                TraitItem::Verbatim(v) => {
                    // Try interpreting the input as an impl item.
                    match syn::parse::<ImplItem>(v.into()).unwrap() {
                        ImplItem::Const(mut c) => {
                            add_docext(&mut c.attrs);
                            c.to_token_stream().into()
                        }
                        ImplItem::Fn(mut f) => {
                            add_docext(&mut f.attrs);
                            f.to_token_stream().into()
                        }
                        ImplItem::Type(mut t) => {
                            add_docext(&mut t.attrs);
                            t.to_token_stream().into()
                        }
                        ImplItem::Macro(mut m) => {
                            add_docext(&mut m.attrs);
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

/// Add KaTeX syntax rendering and image support for doc comments in the given
/// item attributes.
fn add_docext(attrs: &mut Vec<Attribute>) {
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

    let doc = add_tex(doc);
    let doc = add_images(doc);

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

// TODO Update comment
/// Enable KaTeX rendering for the doc comment in the given item attributes.
fn add_tex(doc: String) -> String {
    // Regex to replace all continuous whitespace (including newlines) with a single
    // space.
    let re = Regex::new(r"\s+").unwrap();

    let doc: String = parser::parse_math(&doc)
        .into_iter()
        .map(|event| match event {
            parser::Event::Text(text) => text.to_owned(),
            parser::Event::Math(math) => {
                // Collapse all newlines and whitespace in math blocks into single spaces and
                // replace the math blocks with <span data-tex="MATH">MATH</span> elements. The
                // reason for this is explained below.
                let math = re.replace_all(math, " ");
                format!(r#"<span class="docext-math" data-tex="{math}">{math}</span>"#)
            }
        })
        .collect();

    // Add the KaTeX CSS and JS to the doc comment, enabling TeX rending. The
    // rendering script first copies the TeX from the data-tex attribute into the
    // inner HTML of the span to ensure that the math is unaffected by
    // markdown rendering. This avoids rendering issues in the output HTML, while
    // still providing mostly decent IDE hovers.
    //
    // (Otherwise, for example starting a line with "-" (minus) in the math block
    // would cause the markdown to render as a list and completely break the math,
    // or for example writing $[a](b)$ would render as a link.)
    //
    // Finally, the script calls `renderMathInElement` on its parent, so that the
    // TeX is only loaded in the doc comment, not the entire page.
    doc + r#"
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/katex.min.css" integrity="sha384-GvrOXuhMATgEsSwCs4smul74iXGOixntILdUW9XmUC6+HX0sLNAK3q71HotJqlAn" crossorigin="anonymous">
<script src="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/katex.min.js" integrity="sha384-cpW21h6RZv/phavutF+AuVYrr+dA8xD9zs6FwLpaCct6O9ctzYFfFr4dgmgccOTx" crossorigin="anonymous"></script>
<script src="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/contrib/auto-render.min.js" integrity="sha384-+VBxd3r6XgURycqtZ117nYw44OOcIax56Z4dCRWbxyPt0Koah1uHoK0o4+/RRE05" crossorigin="anonymous"></script>
<script>
    var d=document;
    var c=d.currentScript;
    var t=c.parentElement.getElementsByClassName("docext-math");
    for(var i=0;i<t.length;i+=1){t[i].innerHTML=t[i].getAttribute("data-tex")};
    d.addEventListener("DOMContentLoaded",function(){
        renderMathInElement(c.parentElement,{
            delimiters:[{left:"$$",right:"$$",display:true},{left:"$",right:"$",display:false}]
        })
    });
</script>"#
}

fn add_images(doc: String) -> String {
    // TODO Properly support when the same image is used multiple times

    // TODO First collect all image paths, then encode them all into spans later,
    // this is more readable
    // Spans containing image data encoded as base64.
    let mut img_spans = String::new();

    for ev in pulldown_cmark::Parser::new(&doc) {
        if let pulldown_cmark::Event::Start(pulldown_cmark::Tag::Image(_, path_or_url, _)) = ev {
            if Url::parse(&path_or_url).is_err() {
                // This is not a URL, so it must be a path to a local image.

                // Load the image relative to the current file.
                let mut path = std::path::PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
                path.push(path_or_url.as_ref());
                let mime = match path.extension() {
                    Some(ext) if ext == "apng" => "image/apng",
                    Some(ext) if ext == "avif" => "image/avif",
                    Some(ext) if ext == "gif" => "image/gif",
                    Some(ext)
                        if ext == "jpg"
                            || ext == "jpeg"
                            || ext == "jfif"
                            || ext == "pjpeg"
                            || ext == "pjp" =>
                    {
                        "image/jpeg"
                    }
                    Some(ext) if ext == "png" => "image/png",
                    Some(ext) if ext == "svg" => "image/svg+xml",
                    Some(ext) if ext == "webp" => "image/webp",
                    Some(ext) if ext == "bmp" => "image/bmp",
                    Some(ext) if ext == "ico" || ext == "cur" => "image/x-icon",
                    Some(ext) if ext == "tif" || ext == "tiff" => "image/tiff",
                    Some(ext) => panic!("Unsupported image format: {}", ext.to_string_lossy()),
                    None => panic!("Image path has no extension: {}", path.to_string_lossy()),
                };
                let metadata = std::fs::metadata(&path)
                    .unwrap_or_else(|_| panic!("Failed to read image: {}", path.to_string_lossy()));
                // TODO Maybe a config for large files
                if metadata.len() > 1024 * 1024 {
                    panic!(
                        "Image file too large: {}, max size is 1MB",
                        path.to_string_lossy()
                    );
                }
                let data = std::fs::read(&path)
                    .unwrap_or_else(|_| panic!("Failed to read image: {}", path.to_string_lossy()));
                let base64 = base64::engine::general_purpose::STANDARD.encode(&data);
                // TODO Split into multiple spans probably, check first if this causes problems
                img_spans.push_str(&format!(
                    r#"<span class="docext-img" data-src="{path_or_url}" data-img="data:{mime};base64,{base64}"></span>"#,
                ));
                img_spans.push('\n');
            }
        }
    }

    if img_spans.is_empty() {
        return doc;
    }

    doc + "\n"
        + &img_spans
        // TODO The script might nee to change as well
        // At least use getElementsWithClassName instead of querySelectorAll
        + r#"
<script>
var elem = document.currentScript.parentElement;
document.addEventListener("DOMContentLoaded",function(){
    elem.querySelectorAll(".docext-img").forEach(function(e){
        console.log(e.getAttribute("data-path"));
        elem.querySelector("img[src='" + e.getAttribute("data-src") + "']").src = e.getAttribute("data-img")
    });
});
</script>
"#
}
