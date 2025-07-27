use tature::Regex;

fn main() {
    println!("Testing .* pattern:");

    let test_cases = [
        ("ab*c", "ac"),  // This works in our tests
        ("ab*c", "abc"), // This works in our tests
        ("a*", ""),      // Isolated star - does this work?
        ("a*", "aaa"),   // Isolated star - does this work?
        (".*", "abc"),   // Dot + star - does this work?
        (".*", ""),      // Dot + star - does this work?
    ];

    for (pattern, text) in test_cases {
        match Regex::new(pattern) {
            Ok(regex) => {
                let result = regex.is_match(text);
                println!("  {pattern} vs '{text}' -> {result}");
            }
            Err(e) => {
                println!("  {pattern} -> ERROR: {e:?}");
            }
        }
    }
}
