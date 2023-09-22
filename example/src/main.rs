#![allow(dead_code)]

use docext::docext;

/// $$1
/// -
/// 2\newline
/// \pi
///
/// $$
#[docext]
pub trait BrokenExample {}

/// $$
/// \{
/// - x
/// $$
#[docext]
pub trait BrokenExample2 {}

/// $$
/// {
/// - x
/// }
/// $$
#[docext]
pub trait BrokenExample3 {}

/// The brace is not closed.
///
/// $$
/// {
/// - x
/// $$
#[docext]
pub trait IntentionallyInvalidTeX {}

/// $$
/// a
/// \\
/// b
/// $$
#[docext]
pub trait BackslashAsNewline {}

fn main() {
    println!("Hello, world!");
}
