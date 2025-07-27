use tature::{Regex, SyntaxFlags};

fn main() {
    println!("Testing groups edge case:");

    let test_cases = [
        ("(ab)+", "ab"),
        ("(ab)+", "abab"),
        ("(ab)+", "ababab"),
        ("(ab)+", "a"), // This should NOT match
        ("(ab)+", "aba"),
    ];

    // First show bytecode for (ab)+
    match Regex::with_syntax("(ab)+", SyntaxFlags::AWK) {
        Ok(regex) => {
            println!("Rust '(ab)+' bytecode ({} bytes): ", regex.buffer.len());
            print!("  ");
            for (i, byte) in regex.buffer.iter().enumerate() {
                print!("{byte:02x}");
                if i < regex.buffer.len() - 1 {
                    print!(" ");
                }
            }
            println!();
            println!("C    '(ab)+' bytecode (18 bytes): 0d 03 00 0b 0b 00 06 01 04 61 04 62 07 01 0a f2 ff 00");
            println!();
        }
        Err(e) => {
            println!("Bytecode debug failed: {e:?}");
        }
    }

    for (pattern, text) in test_cases {
        match Regex::with_syntax(pattern, SyntaxFlags::AWK) {
            Ok(regex) => {
                let result = regex.is_match(text);
                println!(
                    "  {} vs '{}' -> {} (C expects: {})",
                    pattern,
                    text,
                    result,
                    if text == "a" { "false" } else { "true" }
                );
            }
            Err(e) => {
                println!("  {pattern} -> ERROR: {e:?}");
            }
        }
    }
}
