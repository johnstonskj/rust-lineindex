/*!
One-line description.

More detailed description, with

# Example

YYYYY

# Features

*/

#![warn(
    unknown_lints,
    // ---------- Stylistic
    absolute_paths_not_starting_with_crate,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    macro_use_extern_crate,
    nonstandard_style, /* group */
    noop_method_call,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    // ---------- Future
    future_incompatible, /* group */
    rust_2021_compatibility, /* group */
    // ---------- Public
    missing_debug_implementations,
    // missing_docs,
    unreachable_pub,
    // ---------- Unsafe
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    // ---------- Unused
    unused, /* group */
)]
#![deny(
    // ---------- Public
    exported_private_dependencies,
    private_in_public,
    // ---------- Deprecated
    anonymous_parameters,
    bare_trait_objects,
    ellipsis_inclusive_range_patterns,
    // ---------- Unsafe
    deref_nullptr,
    drop_bounds,
    dyn_drop,
)]

use std::{borrow::Cow, ops::RangeInclusive};

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct IndexedString<'a> {
    source: Cow<'a, str>,
    lines: Vec<Range>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Range {
    start: Index,
    end: Index,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index {
    byte: usize,
    character: usize,
}

// ------------------------------------------------------------------------------------------------
// Public Functions
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl<'a> From<&'a str> for IndexedString<'a> {
    fn from(s: &'a str) -> Self {
        let lines = Self::make_lines(s);
        Self {
            source: Cow::Borrowed(s),
            lines,
        }
    }
}

impl From<String> for IndexedString<'_> {
    fn from(s: String) -> Self {
        let lines = Self::make_lines(&s);
        Self {
            source: Cow::Owned(s),
            lines,
        }
    }
}

impl IndexedString<'_> {
    fn make_lines(s: &str) -> Vec<Range> {
        let mut lines: Vec<Range> = Default::default();
        if !s.is_empty() {
            let mut start = Index {
                byte: 0,
                character: 0,
            };
            let mut next = false;
            let end = s.len() - 1;
            for (c_i, (b_i, c)) in s.char_indices().enumerate() {
                if next {
                    let here = Index {
                        byte: b_i,
                        character: c_i,
                    };
                    start = here;
                    next = false;
                }
                if c == '\n' || c_i == end {
                    let here = Index {
                        byte: b_i,
                        character: c_i,
                    };
                    lines.push(Range { start, end: here });
                    next = true;
                }
            }
        }
        lines
    }

    pub fn lines(&self) -> usize {
        self.lines.len()
    }

    pub fn line_for_byte(&self, byte: usize) -> Option<usize> {
        self.line_for(true, byte)
    }

    pub fn line_for_char(&self, character: usize) -> Option<usize> {
        self.line_for(false, character)
    }

    fn line_for(&self, byte: bool, index: usize) -> Option<usize> {
        let start = 0;
        let end = self.lines.len();
        self.inner_line_for(byte, index, start, end)
    }

    fn inner_line_for(&self, byte: bool, index: usize, start: usize, end: usize) -> Option<usize> {
        let mid_index = start + ((end - start) / 2);
        let mid_range = self.lines.get(mid_index).unwrap();
        let mid_range = if byte {
            mid_range.bytes()
        } else {
            mid_range.characters()
        };
        if mid_range.contains(&index) {
            Some(mid_index)
        } else if mid_index > start && index < *mid_range.start() {
            self.inner_line_for(byte, index, start, mid_index - 1)
        } else if mid_index < end && index > *mid_range.end() {
            self.inner_line_for(byte, index, mid_index + 1, end)
        } else {
            None
        }
    }

    pub fn byte_range_for_line(&self, line: usize) -> Option<RangeInclusive<usize>> {
        if let Some(range) = self.lines.get(line) {
            Some(range.bytes())
        } else {
            None
        }
    }

    pub fn char_range_for_line(&self, line: usize) -> Option<RangeInclusive<usize>> {
        if let Some(range) = self.lines.get(line) {
            Some(range.characters())
        } else {
            None
        }
    }

    pub fn line_str(&self, line: usize) -> Option<&str> {
        if let Some(range) = self.byte_range_for_line(line) {
            Some(&self.source[range])
        } else {
            None
        }
    }
}

// ------------------------------------------------------------------------------------------------

impl Range {
    pub fn new(start: Index, end: Index) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> Index {
        self.start
    }

    pub fn end(&self) -> Index {
        self.end
    }

    pub fn bytes(&self) -> RangeInclusive<usize> {
        self.start.byte..=self.end.byte
    }

    pub fn characters(&self) -> RangeInclusive<usize> {
        self.start.character..=self.end.character
    }
}

// ------------------------------------------------------------------------------------------------

impl Index {
    pub fn new(byte: usize, character: usize) -> Self {
        Self { byte, character }
    }

    pub fn byte(&self) -> usize {
        self.byte
    }

    pub fn character(&self) -> usize {
        self.character
    }
}

// ------------------------------------------------------------------------------------------------
// Modules
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Unit Tests
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;


    #[test]
    fn test_empty_string() {
        let indexed = IndexedString::from("");

        println!("{:#?}", indexed);
        assert_eq!(indexed.lines(), 0);
    }

    #[test]
    fn test_simple_lines() {
        let lines = "aa\nbbb\ncccc\ndd";
        let indexed = IndexedString::from(lines);

        println!("{:#?}", indexed);
        assert_eq!(indexed.lines(), 4);

        [
            (0, 0), (1, 0), (2, 0),
            (3, 1), (4, 1), (5, 1), (6, 1),
            (7, 2), (8, 2), (9, 2), (10, 2), (11, 2),
            (12, 3), (13, 3),
        ]
            .into_iter()
            .for_each(|(byte, line)| assert_eq!(indexed.line_for_byte(byte).unwrap(), line));

        [
            (0, "aa\n"),
            (1, "bbb\n"),
            (2, "cccc\n"),
            (3, "dd"),
        ]
            .into_iter()
            .for_each(|(line, string)| assert_eq!(indexed.line_str(line), Some(string)));
    }
}
