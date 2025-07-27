//! Port of the classic regexpr.c regex engine to Rust
//!
//! This is a faithful port of the regex engine used in LambdaMOO, originally written by
//! Tatu Ylonen. It maintains compatibility with the original regex syntax while adding
//! UTF-8 support and tick-based execution limits.
//!
//! Both patterns and input text are processed as UTF-8 characters.

pub mod compiler;
pub mod error;
pub mod matcher;
pub mod opcodes;
pub mod syntax;

pub use error::{RegexError, Result};
pub use syntax::SyntaxFlags;

/// Maximum number of capture groups supported
pub const RE_NREGS: usize = 100;

/// A compiled regular expression pattern
#[derive(Debug, Clone)]
pub struct Regex {
    /// Compiled bytecode buffer
    pub buffer: Vec<u8>,
    /// Translation table for case-insensitive matching
    translate: Option<std::collections::HashMap<char, char>>,
    /// Syntax flags used during compilation
    #[allow(dead_code)]
    syntax: SyntaxFlags,
}

/// Match result with capture group positions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Captures {
    /// Start and end positions of capture groups
    groups: [(Option<usize>, Option<usize>); RE_NREGS],
}

/// Configuration for regex execution limits
#[derive(Debug, Clone, Copy)]
pub struct ExecLimits {
    /// Maximum number of execution steps before timeout
    pub max_ticks: Option<usize>,
    /// Maximum failure stack depth
    pub max_failures: usize,
}

impl Default for ExecLimits {
    fn default() -> Self {
        Self {
            max_ticks: None,
            max_failures: 100_000,
        }
    }
}

impl Regex {
    /// Compile a regex pattern with default syntax
    pub fn new(pattern: &str) -> Result<Self> {
        Self::with_syntax(pattern, SyntaxFlags::default())
    }

    /// Compile a regex pattern with specific syntax flags
    pub fn with_syntax(pattern: &str, syntax: SyntaxFlags) -> Result<Self> {
        // Pattern is already validated as UTF-8 by Rust's &str type
        let regex = compiler::compile(pattern, syntax)?;
        Ok(regex)
    }

    /// Test if the pattern matches anywhere in the text
    pub fn is_match(&self, text: &str) -> bool {
        self.is_match_with_limits(text, ExecLimits::default())
    }

    /// Test if pattern matches with execution limits
    pub fn is_match_with_limits(&self, text: &str, limits: ExecLimits) -> bool {
        self.find_with_limits(text, limits).is_some()
    }

    /// Find the first match in the text
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        self.find_with_limits(text, ExecLimits::default())
    }

    /// Find first match with execution limits
    pub fn find_with_limits(&self, text: &str, limits: ExecLimits) -> Option<(usize, usize)> {
        if let Some(captures) = self.captures_with_limits(text, limits) {
            captures.get(0)
        } else {
            None
        }
    }

    /// Get all capture groups from the first match
    pub fn captures(&self, text: &str) -> Option<Captures> {
        self.captures_with_limits(text, ExecLimits::default())
    }

    /// Get captures with execution limits
    pub fn captures_with_limits(&self, text: &str, limits: ExecLimits) -> Option<Captures> {
        // Text is already validated as UTF-8 by Rust's &str type
        matcher::search(self, text, 0, text.chars().count() as i32, limits)
            .ok()
            .and_then(|pos| {
                if pos >= 0 {
                    matcher::match_at(self, text, pos as usize, limits)
                        .ok()
                        .flatten()
                } else {
                    None
                }
            })
    }
}

impl Captures {
    /// Get the bounds of a capture group
    pub fn get(&self, index: usize) -> Option<(usize, usize)> {
        if index < RE_NREGS {
            if let (Some(start), Some(end)) = self.groups[index] {
                Some((start, end))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get number of capture groups (including group 0)
    pub fn len(&self) -> usize {
        self.groups
            .iter()
            .position(|(start, end)| start.is_none() && end.is_none())
            .unwrap_or(RE_NREGS)
    }

    /// Check if no captures were found
    pub fn is_empty(&self) -> bool {
        self.groups[0].0.is_none()
    }
}
