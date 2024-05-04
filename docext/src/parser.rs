use std::ops::Range;

/// Parse the given input into sequences of text and math blocks.
///
/// This is implemented based on the [KaTeX auto-render script](https://github.com/KaTeX/KaTeX/blob/4f1d9166749ca4bd669381b84b45589f1500a476/contrib/auto-render/splitAtDelimiters.js).
pub fn parse_math(mut text: &str) -> Vec<Event<'_>> {
    let mut events = Vec::new();
    loop {
        if text.is_empty() {
            return events;
        }

        // Find the start of the math block.
        let Some(start) = text.find('$') else {
            // There are no more math blocks.
            events.push(Event::Text(text));
            return events;
        };

        let delim = if text[start..].starts_with("$$") {
            "$$"
        } else {
            "$"
        };
        match find_math_end(text, delim, start) {
            Some(end) => {
                // Push the text before the math block, if there is any.
                if start != 0 {
                    events.push(Event::Text(&text[..start]));
                }
                events.push(Event::Math(&text[start..end], start..end));
                text = &text[end..];
            }
            None => {
                // There is no closing delimiter, so there is no math block.
                events.push(Event::Text(text));
                return events;
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Event<'a> {
    Text(&'a str),
    Math(&'a str, Range<usize>),
}

/// Find the end of the math block, while respecting braces. Return the char
/// index pointing after the end of the closing delimiter of the math block.
fn find_math_end(text: &str, delim: &str, start: usize) -> Option<usize> {
    let start = start + delim.len();
    let mut chars = text[start..].char_indices().fuse().peekable();
    let mut depth = 0;
    while let Some((i, c)) = chars.next() {
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
        } else if c == '\\' {
            // Skip the next character, since it is escaped.
            chars.next();
        } else if c == '$' && depth <= 0 {
            // Might be the end of the math block.

            if delim == "$" {
                return Some(start + i + 1);
            }

            // The delim is "$$", so check that the next character is also $.
            if let Some((i, '$')) = chars.peek() {
                return Some(start + *i + 1);
            }
        }
    }
    None
}

#[cfg(test)]
mod test {
    //! Tests for the parser inspired by the [KaTeX auto-render tests](https://github.com/KaTeX/KaTeX/blob/4f1d9166749ca4bd669381b84b45589f1500a476/contrib/auto-render/test/auto-render-spec.js).

    #[derive(Debug, PartialEq, Eq)]
    enum Event<'a> {
        Text(&'a str),
        Math(&'a str),
    }

    fn parse_math(text: &str) -> Vec<Event> {
        super::parse_math(text)
            .into_iter()
            .map(|event| match event {
                super::Event::Text(text) => Event::Text(text),
                super::Event::Math(math, _) => Event::Math(math),
            })
            .collect()
    }

    /// Doesn't parse math if there are no math block delimiters.
    #[test]
    fn no_delimiters() {
        assert_eq!(parse_math("hello"), vec![Event::Text("hello")]);
    }

    /// Doesn't parse math if there is only an opening delimiter without
    /// a closing delimiter.
    #[test]
    fn one_left_delimiter() {
        assert_eq!(
            parse_math("hello $ world"),
            vec![Event::Text("hello $ world")]
        );
        assert_eq!(
            parse_math("hello $$ world"),
            vec![Event::Text("hello $$ world")]
        );
    }

    /// Parses math if there is a single math block and nothing else.
    #[test]
    fn single_block() {
        assert_eq!(parse_math("$ hello $"), vec![Event::Math("$ hello $"),]);
        assert_eq!(parse_math("$$ hello $$"), vec![Event::Math("$$ hello $$"),]);
    }

    /// Parses math if there are matching delimiters, which constitute a
    /// math block.
    #[test]
    fn matching_delimiters() {
        assert_eq!(
            parse_math("hello $ world $ boo"),
            vec![
                Event::Text("hello "),
                Event::Math("$ world $"),
                Event::Text(" boo")
            ]
        );
        assert_eq!(
            parse_math("hello $$ world $$ boo"),
            vec![
                Event::Text("hello "),
                Event::Math("$$ world $$"),
                Event::Text(" boo")
            ]
        );
    }

    /// Parses multiple math blocks.
    #[test]
    fn multiple_math_blocks() {
        assert_eq!(
            parse_math("hello $ world $ boo $ more $ stuff"),
            vec![
                Event::Text("hello "),
                Event::Math("$ world $"),
                Event::Text(" boo "),
                Event::Math("$ more $"),
                Event::Text(" stuff")
            ]
        );
        assert_eq!(
            parse_math("hello $$ world $$ boo $$ more $$ stuff"),
            vec![
                Event::Text("hello "),
                Event::Math("$$ world $$"),
                Event::Text(" boo "),
                Event::Math("$$ more $$"),
                Event::Text(" stuff")
            ]
        );
    }

    /// Doesn't parse math if there's only one delimiter.
    #[test]
    fn closed_and_unclosed_blocks() {
        assert_eq!(
            parse_math("hello $ world $ boo $ left"),
            vec![
                Event::Text("hello "),
                Event::Math("$ world $"),
                Event::Text(" boo $ left"),
            ]
        );
        assert_eq!(
            parse_math("hello $$ world $$ boo $$ left"),
            vec![
                Event::Text("hello "),
                Event::Math("$$ world $$"),
                Event::Text(" boo $$ left"),
            ]
        );
        assert_eq!(
            parse_math("hello $$ world $$ boo $ left"),
            vec![
                Event::Text("hello "),
                Event::Math("$$ world $$"),
                Event::Text(" boo $ left"),
            ]
        );
    }

    /// Ignores delimiters in braces.
    #[test]
    fn delimiters_in_braces() {
        assert_eq!(
            parse_math("hello $ world { $ } boo $ left"),
            vec![
                Event::Text("hello "),
                Event::Math("$ world { $ } boo $"),
                Event::Text(" left"),
            ]
        );
        assert_eq!(
            parse_math("hello $$ world { $$ } boo $$ left"),
            vec![
                Event::Text("hello "),
                Event::Math("$$ world { $$ } boo $$"),
                Event::Text(" left"),
            ]
        );

        assert_eq!(
            parse_math("hello $ world { { } $ } boo $ left"),
            vec![
                Event::Text("hello "),
                Event::Math("$ world { { } $ } boo $"),
                Event::Text(" left"),
            ]
        );
        assert_eq!(
            parse_math("hello $$ world { { } $$ } boo $$ left"),
            vec![
                Event::Text("hello "),
                Event::Math("$$ world { { } $$ } boo $$"),
                Event::Text(" left"),
            ]
        );
    }

    /// Processes consecutive sequences of $.
    #[test]
    fn consecutive_dollar_sequences() {
        assert_eq!(
            parse_math("$hello$$world$$boo$foo$bar$"),
            vec![
                Event::Math("$hello$"),
                Event::Math("$world$"),
                Event::Math("$boo$"),
                Event::Text("foo"),
                Event::Math("$bar$"),
            ]
        );
    }

    /// Ignores \$.
    #[test]
    fn ignores_escaped_dollars() {
        assert_eq!(
            parse_math("$x = \\$hello$"),
            vec![Event::Math("$x = \\$hello$")]
        );
        assert_eq!(
            parse_math("$$x = \\$$hello$$"),
            vec![Event::Math("$$x = \\$$hello$$")]
        );
        assert_eq!(
            parse_math("$$x = \\$\\$hello$$"),
            vec![Event::Math("$$x = \\$\\$hello$$")]
        );
    }

    /// Doesn't get confused when $ and $$ are mixed.
    #[test]
    fn dollars_mix() {
        assert_eq!(
            parse_math("$hello$world$$boo$$"),
            vec![
                Event::Math("$hello$"),
                Event::Text("world"),
                Event::Math("$$boo$$"),
            ]
        );
        assert_eq!(
            parse_math("$hello$$world$$$boo$$"),
            vec![
                Event::Math("$hello$"),
                Event::Math("$world$"),
                Event::Math("$$boo$$"),
            ]
        );
    }
}
