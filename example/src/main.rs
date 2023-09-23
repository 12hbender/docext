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

/// $\pi$
///
/// Should not be rendered as a link:
///
/// $$
/// [a](b)
/// $$
///
/// Inline:
///
/// Hello $[a](b)$ world.
///
/// Hello $\int_0^{\infty}, \sigma = \pi$ world.
#[docext]
pub trait LinkGoneWrong {}

fn main() {
    println!("Hello, world!");
}
