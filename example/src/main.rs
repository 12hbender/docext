#![allow(dead_code)]

use docext::docext;

/// $$
///
/// 1
/// -
/// 2
///
/// $$
#[docext]
pub trait BrokenExample {}

fn main() {
    println!("Hello, world!");
}
