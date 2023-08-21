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

    /// Hello?
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
