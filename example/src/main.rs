#![allow(dead_code)]

use docext::docext;

/// My f. I like it.
///
/// $$\pi$$
#[docext]
pub mod f {
    use docext::docext;

    /// My trait T. Does not use docext.
    ///
    /// $$\frac{1}{2}$$
    #[docext]
    pub trait T {
        /// My other f. $\phi$
        #[docext]
        fn lalalalalalalalallalalalalalalllal();
    }

    /// This needs to be fixed. TODO Idea: the script with calls
    /// renderMathInElement should first set the parent's innerHTML to the
    /// parent's text. Does that make sense? No, that can't work
    ///
    /// TODO Could I do something like this: have specific <data-tex-block> and
    /// <data-tex> tags? And then render math in those tags only? Or maybe a div
    /// with a specific attribute? Not very ergonomic
    ///
    /// TODO Alternatively, since I will need to parse markdown anyway, maybe
    /// just use something like the katex crate to pre-render the maths. Would
    /// be better to do the wasm bindings myself later. In fact I might need to
    /// do that immediately since I'd like to use renderMathInElement.
    ///
    /// The nice part about this is that I also don't need to inline KaTeX or
    /// include any specific scripts. The not-so-nice part is that I have no
    /// idea how this will render inline.
    #[docext]
    pub trait BrokenExample {}

    /// Hello?
    ///
    /// <details>
    /// <summary>Click to expand</summary>
    /// Hello
    /// </details>
    #[docext]
    pub struct S;

    /// Hey OK?
    #[docext]
    impl S {
        /// Hey OK OK? $\alpha$
        #[docext]
        pub fn f() {}
    }
}

fn main() {
    println!("Hello, world!");
}
