use tature::{Regex, SyntaxFlags};

fn main() {
    println!("Testing Rust hex escape \\x41");

    match Regex::with_syntax("\\x41", SyntaxFlags::ANSI_HEX) {
        Ok(regex) => {
            println!("Compile succeeded, buffer length: {}", regex.buffer.len());
            println!("Bytecode: {:02x?}", regex.buffer);

            println!("Testing matches:");
            println!("  'ABC' -> {}", regex.is_match("ABC"));
            println!("  'A' -> {}", regex.is_match("A"));
            println!("  'BCD' -> {}", regex.is_match("BCD"));
        }
        Err(e) => {
            println!("Compile failed: {e}");
        }
    }
}
