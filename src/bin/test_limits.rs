use regexpr::{ExecLimits, Regex};

fn main() {
    println!("Testing execution limits:");

    let regex = Regex::new("a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*").unwrap();
    let text = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa!"; // Doesn't match - will cause backtracking

    println!("Pattern: a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*");
    println!("Text: {text}");
    println!("Text length: {}", text.len());

    // Test with no limits first
    println!("\nTesting with no limits:");
    let start = std::time::Instant::now();
    let result_unlimited = regex.is_match(text);
    let duration_unlimited = start.elapsed();
    println!("Result: {result_unlimited} (took {duration_unlimited:?})");

    // Test with very low limits
    let limits = ExecLimits {
        max_ticks: Some(100), // Very low limit
        max_failures: 50,     // Low limit
    };

    println!("\nTesting with limits (max_ticks: 100, max_failures: 50):");
    let start = std::time::Instant::now();
    let result_limited = regex.is_match_with_limits(text, limits);
    let duration_limited = start.elapsed();
    println!("Result: {result_limited} (took {duration_limited:?})");

    if duration_limited < duration_unlimited {
        println!("✅ Limits appear to be working (faster execution)");
    } else {
        println!("❌ Limits may not be working properly");
    }

    // Test what the test expects
    let test_limits = ExecLimits {
        max_ticks: Some(1000),
        max_failures: 100,
    };

    println!("\nTesting with test limits (max_ticks: 1000, max_failures: 100):");
    let result_test = regex.is_match_with_limits(text, test_limits);
    println!("Result: {result_test} (test expects: false)");

    if !result_test {
        println!("✅ Test should pass");
    } else {
        println!("❌ Test will fail - limits not hit or pattern matches");
    }
}
