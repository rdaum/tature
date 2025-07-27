# tature

A Rust port of the classic early-90s regular expression engine used in LambdaMOO.

[LambdaMOO](https://en.wikipedia.org/wiki/LambdaMOO), and my Rust recreation [mooR](https://github.com/rdaum/moor/),
depends on an ancient regex implementation that predates PCRE. While it's possible to replicate its behavior
with [oniguruma](https://github.com/kkos/oniguruma), that added a substantial compile time C dependency to my program
I wasn't entirely happy with, and I was never sure I was getting full compatibility.

So this crate tries to reproduce all quirks and features of the [original implementation](regexpr.c) in memory-safe Rust, 
with  added tick-based execution limits to prevent runaway regex execution in sandboxed environments.

**Note**: This port was created almost entirely by Claude Code through automated conversion and comparison with the
original C code. So it's also a bit of an example (good and bad) of what is possible in that way. It is *quite* possible
it has broken behaviours.

## Features

- **Syntax Compatibility** - Supports the exact regex syntax used in LambdaMOO
- **Multiple Syntax Modes** - EMACS, AWK, GREP, and EGREP styles
- **Tick-based Execution Limits** - Prevent runaway regex execution
- **UTF-8 Support** - Full Unicode character processing (not just ASCII)
- **Memory Safe** - No buffer overflows or memory leaks
- **Capture Groups** - Full support for backreferences and capture groups

## Quick Start

```rust
use regexpr::{Regex, SyntaxFlags, ExecLimits};

// Basic matching
let regex = Regex::new("hello.*world").unwrap();
assert!(regex.is_match("hello beautiful world"));

// Using different syntax modes
let regex = Regex::with_syntax("(foo|bar)+", SyntaxFlags::AWK).unwrap();
assert!(regex.is_match("foobar"));

// With execution limits to prevent DoS
let limits = ExecLimits {
max_ticks: Some(10000),
max_failures: 1000,
};
let regex = Regex::new("a*a*a*a*").unwrap();
assert!(!regex.is_match_with_limits("aaaaaaaaaa!", limits)); // Prevents catastrophic backtracking
```

## Syntax Modes

### EMACS Style (Default)

The traditional EMACS regex syntax - parentheses and vertical bar require backslashes:

```rust
let regex = Regex::with_syntax("\\(foo\\|bar\\)", SyntaxFlags::EMACS).unwrap();
```

### AWK Style

More modern syntax - no backslashes needed for grouping:

```rust
let regex = Regex::with_syntax("(foo|bar)", SyntaxFlags::AWK).unwrap();
```

### GREP Style

Traditional GREP syntax - plus and question mark require backslashes:

```rust
let regex = Regex::with_syntax("ab\\+c", SyntaxFlags::GREP).unwrap();
```

## Supported Regex Features

### Basic Patterns

- `.` - Any character except newline
- `*` - Zero or more of previous
- `^` - Beginning of line
- `$` - End of line
- `[abc]` - Character sets
- `[^abc]` - Negated character sets
- `[a-z]` - Character ranges

### Extended Features (when enabled)

- `+` - One or more (AWK/GREP syntax dependent)
- `?` - Zero or one (AWK/GREP syntax dependent)
- `|` - Alternation (syntax dependent)
- `()` - Grouping (syntax dependent)
- `\1, \2, ...` - Backreferences
- `\w, \W` - Word/non-word characters
- `\b, \B` - Word boundaries
- `\<, \>` - Word start/end
- `\`, \'' - Buffer start/end

### ANSI Escape Sequences (with ANSI_HEX flag)

- `\n, \t, \r` - Newline, tab, carriage return
- `\xHH` - Hexadecimal character codes

## Execution Limits

Prevent regex-based DoS attacks with configurable limits:

```rust
use regexpr::ExecLimits;

let limits = ExecLimits {
max_ticks: Some(50000),     // Maximum execution steps
max_failures: 10000,        // Maximum backtrack points
};

// This will timeout instead of running forever
let result = regex.is_match_with_limits("aaaaaaaaaab", limits);
```

## Capture Groups

Extract matched subgroups:

```rust
let regex = Regex::with_syntax(r"(\w+)\s+(\d+)", SyntaxFlags::AWK).unwrap();
if let Some(captures) = regex.captures("hello 123") {
let full_match = captures.get(0).unwrap();    // (0, 9) "hello 123"
let word = captures.get(1).unwrap();          // (0, 5) "hello"  
let number = captures.get(2).unwrap();        // (6, 9) "123"
}
```

## Differences from Modern Regex Engines

This engine implements the **classical** regex syntax from the 1990s, which differs from modern PCRE/Perl regex in
several ways:

- No lazy quantifiers (`*?`, `+?`)
- No lookahead/lookbehind assertions
- Different escape sequence handling
- Simpler quantifier behavior

## Use Cases

This engine might be useful when you need:

- **Exact compatibility** with legacy regex behavior
- **Predictable performance** with execution limits
- **Simple, classical regex syntax**

But it's also probably completely pointless for most people.

## License

This code is derived from the original [`regexpr.c`](regexpr.c) by Tatu Ylonen, modified for LambdaMOO by Pavel Curtis
in 1995. The license follows the same permissive terms as the original:

```
regexpr.c
Author: Tatu Ylonen <ylo@ngs.fi>
Copyright (c) 1991 Tatu Ylonen, Espoo, Finland

Permission to use, copy, modify, distribute, and sell this software
and its documentation for any purpose is hereby granted without fee,
provided that the above copyright notice appear in all copies.
```
