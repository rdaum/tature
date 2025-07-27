use regexpr::{Regex, SyntaxFlags};

fn main() {
    let patterns = vec![
        "a", "ab*c", "ab+c",  // AWK mode should work
        "(ab)+", // AWK mode grouping
        "hello", "[abc]",
    ];

    for pattern in patterns {
        println!("=== Pattern: '{pattern}' ===");

        let result = if pattern == "ab+c" || pattern == "(ab)+" {
            Regex::with_syntax(pattern, SyntaxFlags::AWK)
        } else {
            Regex::new(pattern)
        };

        match result {
            Ok(regex) => {
                println!("Compiled successfully");
                println!("Buffer length: {}", regex.buffer.len());
                println!("Bytecode:");

                let mut pos = 0;
                while pos < regex.buffer.len() {
                    let opcode = regex.buffer[pos];
                    print!("{pos:3}: {opcode:02x} ");

                    match opcode {
                        0 => println!("End"),
                        1 => println!("Bol"),
                        2 => println!("Eol"),
                        3 => {
                            println!("Set");
                            pos += 32; // Skip character set
                        }
                        4 => {
                            pos += 1;
                            if pos < regex.buffer.len() {
                                println!("Exact '{}'", regex.buffer[pos] as char);
                            }
                        }
                        5 => println!("AnyChar"),
                        6 => {
                            pos += 1;
                            if pos < regex.buffer.len() {
                                println!("StartMemory {}", regex.buffer[pos]);
                            }
                        }
                        7 => {
                            pos += 1;
                            if pos < regex.buffer.len() {
                                println!("EndMemory {}", regex.buffer[pos]);
                            }
                        }
                        8 => {
                            pos += 1;
                            if pos < regex.buffer.len() {
                                println!("MatchMemory {}", regex.buffer[pos]);
                            }
                        }
                        9 => {
                            pos += 1;
                            let mut disp = 0i16;
                            if pos + 1 < regex.buffer.len() {
                                disp = regex.buffer[pos] as i16
                                    | ((regex.buffer[pos + 1] as i16) << 8);
                                pos += 1;
                            }
                            println!("Jump {}", pos as i32 + disp as i32 + 1);
                        }
                        10 => {
                            pos += 1;
                            let mut disp = 0i16;
                            if pos + 1 < regex.buffer.len() {
                                disp = regex.buffer[pos] as i16
                                    | ((regex.buffer[pos + 1] as i16) << 8);
                                pos += 1;
                            }
                            println!("StarJump {}", pos as i32 + disp as i32 + 1);
                        }
                        11 => {
                            pos += 1;
                            let mut disp = 0i16;
                            if pos + 1 < regex.buffer.len() {
                                disp = regex.buffer[pos] as i16
                                    | ((regex.buffer[pos + 1] as i16) << 8);
                                pos += 1;
                            }
                            println!("FailureJump {}", pos as i32 + disp as i32 + 1);
                        }
                        12 => {
                            pos += 1;
                            let mut disp = 0i16;
                            if pos + 1 < regex.buffer.len() {
                                disp = regex.buffer[pos] as i16
                                    | ((regex.buffer[pos + 1] as i16) << 8);
                                pos += 1;
                            }
                            println!("UpdateFailureJump {}", pos as i32 + disp as i32 + 1);
                        }
                        13 => {
                            pos += 1;
                            let mut disp = 0i16;
                            if pos + 1 < regex.buffer.len() {
                                disp = regex.buffer[pos] as i16
                                    | ((regex.buffer[pos + 1] as i16) << 8);
                                pos += 1;
                            }
                            println!("DummyFailureJump {}", pos as i32 + disp as i32 + 1);
                        }
                        _ => println!("Unknown opcode"),
                    }
                    pos += 1;
                }

                // Test the pattern
                println!("Test results:");
                if pattern == "(ab)+" {
                    println!("  'ab' -> {}", regex.is_match("ab"));
                    println!("  'abab' -> {}", regex.is_match("abab"));
                    println!("  'ababab' -> {}", regex.is_match("ababab"));
                    println!("  'a' -> {}", regex.is_match("a"));
                    println!("  'aba' -> {}", regex.is_match("aba"));
                } else if pattern == "ab+c" {
                    println!("  'ac' -> {} (should be false)", regex.is_match("ac"));
                    println!("  'abc' -> {} (should be true)", regex.is_match("abc"));
                    println!("  'abbc' -> {} (should be true)", regex.is_match("abbc"));
                    println!("  'a' -> {} (should be false)", regex.is_match("a"));
                    println!("  'ab' -> {} (should be false)", regex.is_match("ab"));
                } else {
                    println!("  'abc' -> {}", regex.is_match("abc"));
                    println!("  'ac' -> {}", regex.is_match("ac"));
                    if pattern == "ab*c" {
                        println!("  'abbc' -> {}", regex.is_match("abbc"));
                    }
                }
            }
            Err(e) => println!("Failed to compile: {e}"),
        }
        println!();
    }
}
