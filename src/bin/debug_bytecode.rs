use tature::Regex;

fn main() {
    let patterns = ["a*", "ab*c", ".*"];

    for pattern in patterns {
        println!("\nPattern: {pattern}");
        match Regex::new(pattern) {
            Ok(regex) => {
                println!("  Bytecode length: {}", regex.buffer.len());
                print!("  Bytecode: ");
                for (i, byte) in regex.buffer.iter().enumerate() {
                    print!("{byte:02x}");
                    if i < regex.buffer.len() - 1 {
                        print!(" ");
                    }
                }
                println!();

                // Test some simple cases
                let test_cases = ["", "a", "aa", "b"];
                for test in test_cases {
                    let result = regex.is_match(test);
                    println!("    vs '{test}' -> {result}");
                }
            }
            Err(e) => {
                println!("  Error: {e:?}");
            }
        }
    }
}
