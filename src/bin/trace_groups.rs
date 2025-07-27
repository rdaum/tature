use tature::{Regex, SyntaxFlags};

fn main() {
    println!("Tracing (ab)+ vs 'a' execution:");

    match Regex::with_syntax("(ab)+", SyntaxFlags::AWK) {
        Ok(regex) => {
            println!("Bytecode (raw): {:02x?}", regex.buffer);
            println!("Bytecode (decoded):");
            println!("  Expected: 0d 03 00 0b 0b 00 06 01 04 61 04 62 07 01 0a f2 ff 00");
            println!("  Position: 00 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15 16 17");
            println!();
            println!("  Pos  Bytes       Instruction");
            println!("  ---  ----------  -----------");
            println!("   0:  0d 03 00    DummyFailureJump -> 6");
            println!("   3:  0b 0b 00    FailureJump -> 17");
            println!("   6:  06 01       StartMemory reg1");
            println!("   8:  04 61       Exact 'a'");
            println!("  10:  04 62       Exact 'b'");
            println!("  12:  07 01       EndMemory reg1");
            println!("  14:  0a f2 ff    StarJump -> 3");
            println!("  17:  00          End");

            println!("\nTesting against 'a':");
            let result = regex.is_match("a");
            println!("Result: {result} (should be false)");
        }
        Err(e) => {
            println!("Error: {e:?}");
        }
    }
}
