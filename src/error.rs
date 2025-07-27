//! Error types for the regex engine

use std::fmt;

/// Result type for regex operations
pub type Result<T> = std::result::Result<T, RegexError>;

/// Errors that can occur during regex compilation or execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegexError {
    /// Pattern compilation failed
    CompileError(String),
    /// Pattern is too complex (too many nested operators)
    TooComplex,
    /// Unmatched parentheses
    UnmatchedParenthesis,
    /// Bad hexadecimal escape sequence
    BadHexEscape,
    /// Invalid back-reference number
    BadBackReference,
    /// Badly placed special character
    BadSpecialChar,
    /// Pattern ends prematurely
    PrematureEnd,
    /// Out of memory during compilation
    OutOfMemory,
    /// Execution timed out (exceeded tick limit)
    Timeout,
    /// Execution failed (stack overflow or other runtime error)
    ExecutionError,
    /// Invalid UTF-8 in input
    InvalidUtf8,
}

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegexError::CompileError(msg) => write!(f, "Regex compilation error: {msg}"),
            RegexError::TooComplex => write!(f, "Regular expression too complex"),
            RegexError::UnmatchedParenthesis => write!(f, "Badly placed parenthesis"),
            RegexError::BadHexEscape => write!(f, "Bad hexadecimal number"),
            RegexError::BadBackReference => write!(f, "Bad match register number"),
            RegexError::BadSpecialChar => write!(f, "Badly placed special character"),
            RegexError::PrematureEnd => write!(f, "Regular expression ends prematurely"),
            RegexError::OutOfMemory => write!(f, "Out of memory"),
            RegexError::Timeout => write!(f, "Regex execution timed out"),
            RegexError::ExecutionError => write!(f, "Regex execution error"),
            RegexError::InvalidUtf8 => write!(f, "Invalid UTF-8 in input"),
        }
    }
}

impl std::error::Error for RegexError {}
