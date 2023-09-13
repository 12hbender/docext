//! Tests for [collapse_math] inspired by the [KaTeX auto-render tests](https://github.com/KaTeX/KaTeX/blob/4f1d9166749ca4bd669381b84b45589f1500a476/contrib/auto-render/test/auto-render-spec.js).

use super::collapse_math;

/// Doesn't collapse the text if there are not math block delimiters.
#[test]
fn no_delimiters() {
    assert_eq!(collapse_math("he\nllo"), "he\nllo");
}

/// Doesn't collapse the text if there is only an opening delimiter without a
/// closing delimiter.
#[test]
fn one_left_delimiter() {
    assert_eq!(collapse_math("he\nllo $ wo\nrld"), "he\nllo $ wo\nrld");
    assert_eq!(collapse_math("he\nllo $$ wo\nrld"), "he\nllo $$ wo\nrld");
}

/// Collapses the text if there are matching delimiters, which constitute a math
/// block.
#[test]
fn matching_delimiters() {
    assert_eq!(
        collapse_math("he\nllo $ wo\nrld $ b\noo"),
        "he\nllo $ wo rld $ b\noo"
    );
    assert_eq!(
        collapse_math("he\nllo $$ wo\nrld $$ b\noo"),
        "he\nllo $$ wo rld $$ b\noo"
    );
}

/// Collapses all math blocks.
#[test]
fn multiple_math_blocks() {
    assert_eq!(
        collapse_math("he\nllo $ wo\nrld $ b\noo $ mo\nre $ stu\nff"),
        "he\nllo $ wo rld $ b\noo $ mo re $ stu\nff"
    );
    assert_eq!(
        collapse_math("he\nllo $$ wo\nrld $$ b\noo $$ mo\nre $$ stu\nff"),
        "he\nllo $$ wo rld $$ b\noo $$ mo re $$ stu\nff"
    );
}

/// Doesn't collapse the ending if there's only one delimiter.
#[test]
fn closed_and_unclosed_blocks() {
    assert_eq!(
        collapse_math("he\nllo $ wo\nrld $ b\noo $ le\nft"),
        "he\nllo $ wo rld $ b\noo $ le\nft"
    );
    assert_eq!(
        collapse_math("he\nllo $$ wo\nrld $$ b\noo $$ le\nft"),
        "he\nllo $$ wo rld $$ b\noo $$ le\nft"
    );
    assert_eq!(
        collapse_math("he\nllo $$ wo\nrld $$ b\noo $ le\nft"),
        "he\nllo $$ wo rld $$ b\noo $ le\nft"
    );
}

/// Ignores delimiters in braces.
#[test]
fn delimiters_in_braces() {
    assert_eq!(
        collapse_math("he\nllo $ wo\nrld { $ } b\noo $ le\nft"),
        "he\nllo $ wo rld { $ } b oo $ le\nft"
    );
    assert_eq!(
        collapse_math("he\nllo $$ wo\nrld { $$ } b\noo $$ le\nft"),
        "he\nllo $$ wo rld { $$ } b oo $$ le\nft"
    );

    assert_eq!(
        collapse_math("he\nllo $ wo\nrld { { } $ } b\noo $ le\nft"),
        "he\nllo $ wo rld { { } $ } b oo $ le\nft"
    );
    assert_eq!(
        collapse_math("he\nllo $$ wo\nrld { { } $$ } b\noo $$ le\nft"),
        "he\nllo $$ wo rld { { } $$ } b oo $$ le\nft"
    );
}

/// Processes consecutive sequences of $.
#[test]
fn since_dollar_sequences() {
    assert_eq!(
        collapse_math("$he\nllo$$wo\nrld$$b\noo$f\noo$b\nar$"),
        "$he llo$$wo rld$$b oo$f\noo$b ar$",
    );
}

/// Ignores \\$.
#[test]
fn ignores_escaped_dollars() {
    assert_eq!(collapse_math("$x = \\$he\nllo$"), "$x = \\$he llo$");
    assert_eq!(collapse_math("$$x = \\$$he\nllo$$"), "$$x = \\$$he llo$$");
    assert_eq!(
        collapse_math("$$x = \\$\\$he\nllo$$"),
        "$$x = \\$\\$he llo$$"
    );
}

/// Doesn't get confused when $ and $$ are mixed.
#[test]
fn dollars_mix() {
    assert_eq!(
        collapse_math("$he\nllo$wo\nrld$$b\noo$$"),
        "$he llo$wo\nrld$$b oo$$"
    );
    assert_eq!(
        collapse_math("$he\nllo$$wo\nrld$$$b\noo$$"),
        "$he llo$$wo rld$$$b oo$$"
    );
}
