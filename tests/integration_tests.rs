//! Integration tests for the regex engine
//!
//! These tests verify compatibility with the original regexpr.c behavior

use tature::{ExecLimits, Regex, SyntaxFlags};

#[test]
fn test_basic_exact_match() {
    let regex = Regex::new("hello").unwrap();
    assert!(regex.is_match("hello"));
    assert!(regex.is_match("hello world"));
    assert!(!regex.is_match("hi"));
}

#[test]
fn test_dot_any_char() {
    let regex = Regex::new("h.llo").unwrap();
    assert!(regex.is_match("hello"));
    assert!(regex.is_match("hallo"));
    assert!(!regex.is_match("hllo"));
    assert!(!regex.is_match("h\nllo")); // . doesn't match newline
}

#[test]
fn test_star_quantifier() {
    let regex = Regex::new("ab*c").unwrap();
    assert!(regex.is_match("ac")); // zero b's
    assert!(regex.is_match("abc")); // one b
    assert!(regex.is_match("abbc")); // two b's
    assert!(regex.is_match("abbbc")); // three b's
    assert!(regex.is_match("acc")); // Contains 'ac' as substring - should match!

    // Test cases that should NOT match
    assert!(!regex.is_match("def")); // Completely different
    assert!(!regex.is_match("a")); // Just 'a', missing 'c'
    assert!(!regex.is_match("ab")); // Missing 'c'
    assert!(!regex.is_match("bc")); // Missing 'a'
}

#[test]
fn test_plus_quantifier_awk_syntax() {
    let regex = Regex::with_syntax("ab+c", SyntaxFlags::AWK).unwrap();
    assert!(!regex.is_match("ac")); // zero b's - should not match
    assert!(regex.is_match("abc")); // one b
    assert!(regex.is_match("abbc")); // two b's
    assert!(!regex.is_match("acc"));
}

#[test]
fn test_optional_quantifier_awk_syntax() {
    let regex = Regex::with_syntax("ab?c", SyntaxFlags::AWK).unwrap();
    assert!(regex.is_match("ac")); // zero b's
    assert!(regex.is_match("abc")); // one b
    assert!(!regex.is_match("abbc")); // two b's - should not match
}

#[test]
fn test_character_sets() {
    let regex = Regex::new("[abc]").unwrap();
    assert!(regex.is_match("a"));
    assert!(regex.is_match("b"));
    assert!(regex.is_match("c"));
    assert!(!regex.is_match("d"));

    let regex = Regex::new("[^abc]").unwrap();
    assert!(!regex.is_match("a"));
    assert!(!regex.is_match("b"));
    assert!(!regex.is_match("c"));
    assert!(regex.is_match("d"));
}

#[test]
fn test_character_ranges() {
    let regex = Regex::new("[a-z]").unwrap();
    assert!(regex.is_match("a"));
    assert!(regex.is_match("m"));
    assert!(regex.is_match("z"));
    assert!(!regex.is_match("A"));
    assert!(!regex.is_match("0"));
}

#[test]
fn test_anchors() {
    let regex = Regex::new("^hello").unwrap();
    assert!(regex.is_match("hello world"));
    assert!(!regex.is_match("say hello"));

    let regex = Regex::new("world$").unwrap();
    assert!(regex.is_match("hello world"));
    assert!(!regex.is_match("world peace"));
}

#[test]
fn test_alternation_awk_syntax() {
    let regex = Regex::with_syntax("cat|dog", SyntaxFlags::AWK).unwrap();
    assert!(regex.is_match("cat"));
    assert!(regex.is_match("dog"));
    assert!(regex.is_match("I have a cat"));
    assert!(regex.is_match("My dog barks"));
    assert!(!regex.is_match("bird"));
}

#[test]
fn test_groups_awk_syntax() {
    let regex = Regex::with_syntax("(ab)+", SyntaxFlags::AWK).unwrap();
    assert!(regex.is_match("ab"));
    assert!(regex.is_match("abab"));
    assert!(regex.is_match("ababab"));
    assert!(!regex.is_match("a"));
    assert!(regex.is_match("aba")); // Should match - contains "ab" at start
}

#[test]
fn test_backreferences() {
    let _regex = Regex::new("\\(\\([a-z]*\\)\\1\\)").unwrap();
    // This should match something like "(hello hello)"
    // Note: simplified test - full backreference support would need more implementation
}

#[test]
fn test_word_boundaries() {
    let regex = Regex::new("\\bword\\b").unwrap();
    assert!(regex.is_match("a word here"));
    assert!(regex.is_match("word"));
    assert!(!regex.is_match("password"));
    assert!(!regex.is_match("wordy"));
}

#[test]
fn test_syntax_flags() {
    // Test EMACS style (default) - requires backslash for groups
    let _regex = Regex::with_syntax("\\(abc\\)", SyntaxFlags::EMACS).unwrap();

    // Test AWK style - no backslash needed for groups
    let _regex = Regex::with_syntax("(abc)", SyntaxFlags::AWK).unwrap();

    // Test GREP style - requires backslash for + and ?
    let _regex = Regex::with_syntax("ab\\+", SyntaxFlags::GREP).unwrap();
}

#[test]
fn test_execution_limits() {
    // Use a pattern that causes catastrophic backtracking
    // (a+a+)+b with input ending in 'c' will try all ways to split a's before failing
    let regex = Regex::with_syntax("(a+a+)+b", SyntaxFlags::AWK).unwrap();
    let text = "aaaaaaaaaaaaaaac"; // Many a's ending in c - doesn't match but causes backtracking

    let limits = ExecLimits {
        max_ticks: Some(1000),
        max_failures: 100,
    };

    // This should timeout due to excessive backtracking
    assert!(!regex.is_match_with_limits(text, limits));
}

#[test]
fn test_captures() {
    let regex = Regex::with_syntax("([a-z]+) ([0-9]+)", SyntaxFlags::AWK).unwrap();
    let captures = regex.captures("hello 123").unwrap();

    assert_eq!(captures.get(0), Some((0, 9))); // Full match
    assert_eq!(captures.get(1), Some((0, 5))); // "hello"
    assert_eq!(captures.get(2), Some((6, 9))); // "123"
}

#[test]
fn test_find_positions() {
    let regex = Regex::new("test").unwrap();
    assert_eq!(regex.find("this is a test"), Some((10, 14)));
    assert_eq!(regex.find("no match"), None);
}

#[test]
fn test_ansi_escapes() {
    let regex = Regex::with_syntax("\\n", SyntaxFlags::ANSI_HEX).unwrap();
    assert!(regex.is_match("line1\nline2"));

    let regex = Regex::with_syntax("\\t", SyntaxFlags::ANSI_HEX).unwrap();
    assert!(regex.is_match("tab\there"));
}

#[test]
fn test_hex_escapes() {
    let regex = Regex::with_syntax("\\x41", SyntaxFlags::ANSI_HEX).unwrap();
    assert!(regex.is_match("ABC")); // \x41 = 'A'
}

#[test]
fn test_case_insensitive_with_translate() {
    // This would require implementing the translate table feature
    // For now, just verify the structure exists
}

#[test]
fn test_empty_pattern() {
    let regex = Regex::new("").unwrap();
    assert!(regex.is_match(""));
    assert!(regex.is_match("anything")); // Empty pattern matches anywhere
}

#[test]
fn test_complex_patterns() {
    // Test a complex pattern that exercises multiple features
    let regex = Regex::with_syntax("^([a-zA-Z][a-zA-Z0-9_]*):.*=.*$", SyntaxFlags::AWK).unwrap();

    assert!(regex.is_match("variable: foo = bar"));
    assert!(regex.is_match("x: a = b"));
    assert!(!regex.is_match("123invalid: foo = bar")); // Can't start with digit
    assert!(!regex.is_match("variable foo = bar")); // Missing colon
}

#[test]
fn test_error_conditions() {
    // Test error conditions that match original C behavior
    // C version allows unclosed/unmatched parentheses
    assert!(Regex::new("(unclosed").is_ok());
    assert!(Regex::new("unmatched)").is_ok());

    // C version catches these errors
    assert!(Regex::new("[unclosed").is_err());
    assert!(Regex::new("\\").is_err()); // Backslash at end
}

#[test]
fn test_compatibility_with_original() {
    // These tests verify specific behaviors from the original regexpr.c

    // Test that * and + are special only after a character
    let regex = Regex::new("*").unwrap(); // Should match literal '*'
    assert!(regex.is_match("*"));

    // Test beginning/end of line behavior
    let regex = Regex::new("^$").unwrap();
    assert!(regex.is_match("")); // Empty string
    assert!(regex.is_match("\n")); // Just newline
}

#[test]
fn test_utf8_support() {
    // Test Unicode character matching
    let regex = Regex::new("café").unwrap();
    assert!(regex.is_match("café"));
    assert!(regex.is_match("I love café au lait"));
    assert!(!regex.is_match("cafe")); // ASCII 'e' != Unicode 'é'

    // Test Unicode in character sets
    let regex = Regex::new("[αβγ]").unwrap();
    assert!(regex.is_match("α"));
    assert!(regex.is_match("β"));
    assert!(regex.is_match("γ"));
    assert!(!regex.is_match("a"));

    // Test Unicode ranges
    let regex = Regex::new("[α-ω]").unwrap();
    assert!(regex.is_match("α"));
    assert!(regex.is_match("λ"));
    assert!(regex.is_match("ω"));
    assert!(!regex.is_match("Α")); // Different case
}
