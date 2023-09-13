//! KaTeX 0.16.8 JavaScript and CSS code.

pub const KATEX_JS: &str = include_str!("dist/katex.min.js");
pub const KATEX_CSS: &str = include_str!("dist/katex.min.css");
pub const AUTO_RENDER_JS: &str = include_str!("dist/auto-render.min.js");

/// JavaScript code to call renderMathInElement on the doc comment.
pub const RENDER_MATH: &str = r#"var c=document.currentScript;document.addEventListener("DOMContentLoaded",function(){renderMathInElement(c.parentElement,{delimiters:[{left:'$$',right:'$$',display:true},{left:'$',right:'$',display:false}]})});"#;
