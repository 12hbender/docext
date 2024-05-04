#![allow(dead_code)]

use docext::docext;

/// Should render as "1 - 2" followed by a pi symbol in the next line.
///
/// $$1
/// -
/// 2\newline
/// \pi
///
/// $$
#[docext]
pub trait BrokenExample {}

/// Should render as "{ -x".
///
/// $$
/// \{
/// - x
/// $$
#[docext]
pub trait BrokenExample2 {}

/// Should render as "-x".
///
/// $$
/// {
/// - x
/// }
/// $$
#[docext]
pub trait BrokenExample3 {}

/// Should not be rendered as math, because the brace is not closed:
///
/// $$
/// {
/// - x
/// $$
#[docext]
pub trait IntentionallyInvalidTeX {}

/// Should contain a newline:
///
/// $$
/// a
/// \\
/// b
/// $$
#[docext]
pub trait BackslashAsNewline {}

/// Hey $\pi$ there
///
/// Should not be rendered as a link:
///
/// $$
/// [a](b)
/// $$
///
/// _Inline_:
///
/// Hello $[a](b)$ world.
///
/// Hello $\int_0^{\infty}, \sigma = \pi$ world.
#[docext]
pub trait LinkGoneWrong {}

/// Hey there
///
/// ![pepega](img/pepega.png)
///
/// Bye there
#[docext]
pub trait Image {}

/// Hey there
///
/// ![pepega](img/pepega.png)
/// ![pepega](img/pepega.png)
/// ![pepega2](img/pepega2.png)
///
/// Bye there
///
/// Some $math$ text.
#[docext]
pub trait Images {}

// TODO Extreme edge case, but should be fixed
// TODO Should probably treat the file as a url...?
/// Hey there
///
/// $$
/// ![pepega](img/pepegas.png)
/// $$
///
/// $$
/// ![pepega](img/pepega.png)
/// $$
///
/// Bye there
//#[docext]
pub trait BrokenImageInTex {}

/// Hey there
///
/// $$
/// x = y = z
/// $$
///
/// Bye there
#[docext]
pub trait TraitWithItems {
    /// Some $a$.
    #[docext]
    const A: usize;
}

/// Hey there $what$ the hell
///
/// What is this $\mathrm{AB}_{16}$ and also $\mathrm{CD}_{16}$ and some
/// $\mathrm{EF}_{16}$.
#[docext]
pub trait BrokenItalics {
    /// Some $a$.
    #[docext]
    const A: usize;
}

/// Math in code block:
/// ```
/// $$
/// This does not get rendered correctly.
/// $$
/// ```
/// Math in inline code: `$This, too.$`
/// Math outside code block: $\pi$
#[docext]
pub trait MathInCode {}

/// Top-level function comment.
/// $$
/// x = \pi
/// $$
#[docext]
fn main() {}
