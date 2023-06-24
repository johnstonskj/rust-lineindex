/*!
A simple line-indexed string.

Rather than destructively breaking a string into lines, this structure will allow
create a vector of byte/character ranges each of which describes a line in the string
original string.

# Example

Given the following simple string,

```text
                    1 1 1 1 1 1 1 1 1 1 2 2 2 2 2 2 2 2 2 2
0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9
-----------------------------------------------------------
a a ␤ b b b ␤ c c c c ␤ d d
```

We get the following set of line index ranges.

| Row  | < Byte | < Char | > Byte | > Char | String  |
|======|========|========|========|========|=========|
| 0    | 0      | 0      | 2      | 2      | "aa␤"   |
| 1    | 3      | 3      | 6      | 6      | "bbb␤"  |
| 2    | 7      | 7      | 11     | 11     | "cccc␤" |
| 3    | 12     | 12     | 13     | 13     | "dd"    |

This set of ranges can be used to determine which line a character is on as well as
returning the indices for a line or even the text of a line.

```rust
use lineindex::IndexedString;

let indexed = IndexedString::from("aa\nbbb\ncccc\ndd");

assert_eq!(indexed.lines(), 4);

assert_eq!(indexed.line_for_byte(4), Some(1));
assert_eq!(indexed.line_for_character(5), Some(1));

assert_eq!(indexed.byte_range_for_line(1), Some(3..=6));
assert_eq!(indexed.character_range_for_line(2), Some(7..=11));

assert_eq!(indexed.line_str(0), Some("aa\n"));
```

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

///
/// This type holds a reference to an string and an index of both the byte and character ranges for
/// lines within the string. Both of these values are immutable, a change to the original string
/// will require construction of a new indexed string.
///
/// The index itself is a vector of start/end indices; `start` is the 0-based index of the first
/// character in the line and `end` is the index of the last character in the line.
///
/// Note that an empty string will result in a zero-length vector of
///
#[derive(Clone, Debug)]
pub struct IndexedString<'a> {
    source: Cow<'a, str>,
    lines: Vec<Range>,
}

///
/// This is a simplified version of [`std::ops::RangeInclusive`] where each end of the range is an
/// [`Index`] structure.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Range {
    start: Index,
    end: Index,
}

///
/// An index value is a tuple of the byte index and character index for a character in the string.
///
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

impl AsRef<str> for IndexedString<'_> {
    fn as_ref(&self) -> &str {
        &self.source
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

    ///
    /// Returns the number of rows calculated within the source string.
    ///
    pub fn lines(&self) -> usize {
        self.lines.len()
    }

    ///
    /// Return a reference to the source, as a string.
    ///
    pub fn as_str(&self) -> &str {
        &self.source
    }

    ///
    /// Return a reference to the source, as a byte array.
    ///
    pub fn as_bytes(&self) -> &[u8] {
        self.source.as_bytes()
    }

    ///
    /// Return the line containing the provided byte index. If the index is
    /// outside the range of the string return `None`.
    ///
    pub fn line_for_byte(&self, byte: usize) -> Option<usize> {
        self.line_for(true, byte)
    }

    ///
    /// Return the line containing the provided character index. If the index is
    /// outside the range of the string return `None`.
    ///
    pub fn line_for_character(&self, character: usize) -> Option<usize> {
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

    ///
    /// Return the byte range (including any terminating newline) for the provided
    /// line number. If the line number is outside the range of the string return `None`.
    ///
    pub fn byte_range_for_line(&self, line: usize) -> Option<RangeInclusive<usize>> {
        self.lines.get(line).map(|range| range.bytes())
    }

    ///
    /// Return the character range (including any terminating newline) for the provided
    /// line number. If the line number is outside the range of the string return `None`.
    ///
    pub fn character_range_for_line(&self, line: usize) -> Option<RangeInclusive<usize>> {
        self.lines.get(line).map(|range| range.characters())
    }

    ///
    /// Return the line, as a string, (including any terminating newline) for the provided
    /// line number. If the line number is outside the range of the string return `None`.
    ///
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
    ///
    /// Construct a new range from the start and end indices.
    ///
    pub fn new(start: Index, end: Index) -> Self {
        Self { start, end }
    }

    ///
    /// Return the start index of this range.
    ///
    pub fn start(&self) -> Index {
        self.start
    }

    ///
    /// Return the end index of this range.
    ///
    pub fn end(&self) -> Index {
        self.end
    }

    ///
    /// Return a standard library range for just the byte indices.
    ///
    pub fn bytes(&self) -> RangeInclusive<usize> {
        self.start.byte..=self.end.byte
    }

    ///
    /// Return a standard library range for just the character indices.
    ///
    pub fn characters(&self) -> RangeInclusive<usize> {
        self.start.character..=self.end.character
    }
}

// ------------------------------------------------------------------------------------------------

impl Index {
    ///
    /// Construct a new index with byte and character indices.
    ///
    pub fn new(byte: usize, character: usize) -> Self {
        Self { byte, character }
    }

    ///
    /// Return the byte part of this index.
    ///
    pub fn byte(&self) -> usize {
        self.byte
    }

    ///
    /// Return the character part of this index.
    ///
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
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 1),
            (4, 1),
            (5, 1),
            (6, 1),
            (7, 2),
            (8, 2),
            (9, 2),
            (10, 2),
            (11, 2),
            (12, 3),
            (13, 3),
        ]
        .into_iter()
        .for_each(|(byte, line)| assert_eq!(indexed.line_for_byte(byte).unwrap(), line));

        [(0, "aa\n"), (1, "bbb\n"), (2, "cccc\n"), (3, "dd")]
            .into_iter()
            .for_each(|(line, string)| assert_eq!(indexed.line_str(line), Some(string)));
    }
}
