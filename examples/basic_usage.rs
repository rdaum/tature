//! Basic usage examples for the regexpr crate

use regexpr::{ExecLimits, Regex, SyntaxFlags};

fn main() {
    println!("=== Basic Pattern Matching ===");
    basic_matching();

    println!("\n=== Syntax Modes ===");
    syntax_modes();

    println!("\n=== Execution Limits ===");
    execution_limits();

    println!("\n=== Capture Groups ===");
    capture_groups();

    println!("\n=== Character Sets and Ranges ===");
    character_sets();

    println!("\n=== Anchoring ===");
    anchoring();
}

fn basic_matching() {
    let regex = Regex::new("hello").unwrap();

    println!("Pattern: 'hello'");
    println!("  'hello world' -> {}", regex.is_match("hello world"));
    println!("  'hi there' -> {}", regex.is_match("hi there"));

    // Dot matches any character except newline
    let regex = Regex::new("h.llo").unwrap();
    println!("\nPattern: 'h.llo'");
    println!("  'hello' -> {}", regex.is_match("hello"));
    println!("  'hallo' -> {}", regex.is_match("hallo"));
    println!("  'h\\nllo' -> {}", regex.is_match("h\nllo"));

    // Star quantifier
    let regex = Regex::new("ab*c").unwrap();
    println!("\nPattern: 'ab*c'");
    println!("  'ac' -> {}", regex.is_match("ac"));
    println!("  'abc' -> {}", regex.is_match("abc"));
    println!("  'abbbbc' -> {}", regex.is_match("abbbbc"));
}

fn syntax_modes() {
    // EMACS style (default) - requires backslashes
    let regex = Regex::with_syntax("\\(foo\\|bar\\)", SyntaxFlags::EMACS).unwrap();
    println!("EMACS style '\\(foo\\|bar\\)':");
    println!("  'foo' -> {}", regex.is_match("foo"));
    println!("  'bar' -> {}", regex.is_match("bar"));

    // AWK style - no backslashes needed
    let regex = Regex::with_syntax("(foo|bar)", SyntaxFlags::AWK).unwrap();
    println!("\nAWK style '(foo|bar)':");
    println!("  'foo' -> {}", regex.is_match("foo"));
    println!("  'bar' -> {}", regex.is_match("bar"));

    // AWK style with plus quantifier
    let regex = Regex::with_syntax("ab+c", SyntaxFlags::AWK).unwrap();
    println!("\nAWK style 'ab+c':");
    println!("  'ac' -> {}", regex.is_match("ac")); // Should be false
    println!("  'abc' -> {}", regex.is_match("abc")); // Should be true
    println!("  'abbc' -> {}", regex.is_match("abbc")); // Should be true

    // GREP style - requires backslashes for + and ?
    let regex = Regex::with_syntax("ab\\+c", SyntaxFlags::GREP).unwrap();
    println!("\nGREP style 'ab\\+c':");
    println!("  'ac' -> {}", regex.is_match("ac"));
    println!("  'abc' -> {}", regex.is_match("abc"));
}

fn execution_limits() {
    // Create a regex that could cause catastrophic backtracking
    let regex = Regex::new("a*a*a*a*a*a*a*a*a*a*").unwrap();
    let text = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaab"; // Doesn't match - will backtrack

    println!("Testing catastrophic backtracking pattern:");
    println!("Pattern: 'a*a*a*a*a*a*a*a*a*a*'");
    println!("Text: 'aaaa...aaab' (30 a's + b)");

    // Without limits - this would take a very long time
    let limits = ExecLimits {
        max_ticks: Some(10000),
        max_failures: 1000,
    };

    let start = std::time::Instant::now();
    let result = regex.is_match_with_limits(text, limits);
    let elapsed = start.elapsed();

    println!("  Result: {result} (completed in {elapsed:?})");
    println!("  -> Execution limits prevented infinite backtracking!");
}

fn capture_groups() {
    let regex = Regex::with_syntax("([a-z]+)\\s+([0-9]+)", SyntaxFlags::AWK).unwrap();
    let text = "hello 123";

    println!("Pattern: '([a-z]+)\\s+([0-9]+)'");
    println!("Text: '{text}'");

    if let Some(captures) = regex.captures(text) {
        println!("Captures:");
        if let Some((start, end)) = captures.get(0) {
            println!(
                "  Group 0 (full match): '{}' at {}-{}",
                &text[start..end],
                start,
                end
            );
        }
        if let Some((start, end)) = captures.get(1) {
            println!(
                "  Group 1 (word): '{}' at {}-{}",
                &text[start..end],
                start,
                end
            );
        }
        if let Some((start, end)) = captures.get(2) {
            println!(
                "  Group 2 (number): '{}' at {}-{}",
                &text[start..end],
                start,
                end
            );
        }
    }
}

fn character_sets() {
    // Basic character set
    let regex = Regex::new("[abc]").unwrap();
    println!("Pattern: '[abc]'");
    println!("  'a' -> {}", regex.is_match("a"));
    println!("  'b' -> {}", regex.is_match("b"));
    println!("  'd' -> {}", regex.is_match("d"));

    // Negated character set
    let regex = Regex::new("[^abc]").unwrap();
    println!("\nPattern: '[^abc]'");
    println!("  'a' -> {}", regex.is_match("a"));
    println!("  'd' -> {}", regex.is_match("d"));

    // Character range
    let regex = Regex::new("[a-z]").unwrap();
    println!("\nPattern: '[a-z]'");
    println!("  'm' -> {}", regex.is_match("m"));
    println!("  'A' -> {}", regex.is_match("A"));
    println!("  '5' -> {}", regex.is_match("5"));
}

fn anchoring() {
    // Beginning of line
    let regex = Regex::new("^hello").unwrap();
    println!("Pattern: '^hello'");
    println!("  'hello world' -> {}", regex.is_match("hello world"));
    println!("  'say hello' -> {}", regex.is_match("say hello"));

    // End of line
    let regex = Regex::new("world$").unwrap();
    println!("\nPattern: 'world$'");
    println!("  'hello world' -> {}", regex.is_match("hello world"));
    println!("  'world peace' -> {}", regex.is_match("world peace"));

    // Both anchors - exact match
    let regex = Regex::new("^hello$").unwrap();
    println!("\nPattern: '^hello$'");
    println!("  'hello' -> {}", regex.is_match("hello"));
    println!("  'hello world' -> {}", regex.is_match("hello world"));
}
