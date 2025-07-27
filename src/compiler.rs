//! Regex pattern compiler
//!
//! This module ports the core compilation logic from re_compile_pattern
//! in regexpr.c:254-721

use crate::{
    error::{RegexError, Result},
    opcodes::{CompiledOp, SyntaxOp},
    syntax::SyntaxFlags,
    Regex, RE_NREGS,
};

/// Maximum nesting level of operators
const MAX_NESTING: usize = 100;
/// Number of precedence levels
const NUM_LEVELS: usize = 5;

/// Compiler state for translating regex patterns to bytecode
struct Compiler {
    /// Input pattern as chars
    pattern: Vec<char>,
    /// Current position in pattern
    pos: usize,
    /// Output bytecode buffer
    buffer: Vec<u8>,
    /// Syntax configuration
    syntax: SyntaxFlags,
    /// Translation table for case conversion (maps char to char)
    translate: Option<std::collections::HashMap<char, char>>,
    /// Operator precedence tables (now use char keys for Unicode support)
    plain_ops: std::collections::HashMap<char, SyntaxOp>,
    quoted_ops: std::collections::HashMap<char, SyntaxOp>,
    precedences: [u8; 256],
    /// Parsing state
    starts: [usize; NUM_LEVELS * MAX_NESTING],
    starts_base: usize,
    future_jumps: [usize; MAX_NESTING],
    num_jumps: usize,
    current_level: usize,
    /// Register tracking
    next_register: u8,
    paren_depth: usize,
    num_open_registers: usize,
    open_registers: [u8; RE_NREGS],
    /// Context state
    beginning_context: bool,
}

impl Compiler {
    /// Create new compiler with given syntax
    fn new(pattern: &str, syntax: SyntaxFlags) -> Result<Self> {
        let pattern_chars: Vec<char> = pattern.chars().collect();

        let mut compiler = Compiler {
            pattern: pattern_chars,
            pos: 0,
            buffer: Vec::new(),
            syntax,
            translate: None,
            plain_ops: std::collections::HashMap::new(),
            quoted_ops: std::collections::HashMap::new(),
            precedences: [0; 256],
            starts: [0; NUM_LEVELS * MAX_NESTING],
            starts_base: 0,
            future_jumps: [0; MAX_NESTING],
            num_jumps: 0,
            current_level: 0,
            next_register: 1,
            paren_depth: 0,
            num_open_registers: 0,
            open_registers: [0; RE_NREGS],
            beginning_context: true,
        };

        compiler.initialize_tables();
        Ok(compiler)
    }

    /// Initialize operator and precedence tables based on syntax flags
    fn initialize_tables(&mut self) {
        // Clear the hash maps
        self.plain_ops.clear();
        self.quoted_ops.clear();

        // Set up memory operators
        for ch in '0'..='9' {
            self.quoted_ops.insert(ch, SyntaxOp::Memory);
        }

        // Quote character
        self.plain_ops.insert('\\', SyntaxOp::Quote);

        // Parentheses
        if self.syntax.needs_backslash_parens() {
            self.quoted_ops.insert('(', SyntaxOp::OpenPar);
            self.quoted_ops.insert(')', SyntaxOp::ClosePar);
        } else {
            self.plain_ops.insert('(', SyntaxOp::OpenPar);
            self.plain_ops.insert(')', SyntaxOp::ClosePar);
        }

        // Vertical bar (alternation)
        if self.syntax.needs_backslash_vbar() {
            self.quoted_ops.insert('|', SyntaxOp::Or);
        } else {
            self.plain_ops.insert('|', SyntaxOp::Or);
        }

        // Star (always unquoted)
        self.plain_ops.insert('*', SyntaxOp::Star);

        // Plus and question mark
        if self.syntax.needs_backslash_plus_qm() {
            self.quoted_ops.insert('+', SyntaxOp::Plus);
            self.quoted_ops.insert('?', SyntaxOp::Optional);
        } else {
            self.plain_ops.insert('+', SyntaxOp::Plus);
            self.plain_ops.insert('?', SyntaxOp::Optional);
        }

        // Newline as alternation
        if self.syntax.newline_or() {
            self.plain_ops.insert('\n', SyntaxOp::Or);
        }

        // Basic operators
        self.plain_ops.insert('[', SyntaxOp::OpenSet);
        self.plain_ops.insert('^', SyntaxOp::Bol);
        self.plain_ops.insert('$', SyntaxOp::Eol);
        self.plain_ops.insert('.', SyntaxOp::AnyChar);

        // GNU extensions
        if !self.syntax.no_gnu_extensions() {
            self.quoted_ops.insert('w', SyntaxOp::WordChar);
            self.quoted_ops.insert('W', SyntaxOp::NotWordChar);
            self.quoted_ops.insert('<', SyntaxOp::WordBeg);
            self.quoted_ops.insert('>', SyntaxOp::WordEnd);
            self.quoted_ops.insert('b', SyntaxOp::WordBound);
            self.quoted_ops.insert('B', SyntaxOp::NotWordBound);
            self.quoted_ops.insert('`', SyntaxOp::BegBuf);
            self.quoted_ops.insert('\'', SyntaxOp::EndBuf);
        }

        // Extended memory
        if self.syntax.ansi_sequences() {
            self.quoted_ops.insert('v', SyntaxOp::ExtendedMemory);
        }

        // Set up precedences
        self.precedences.fill(4); // Default precedence

        if self.syntax.tight_vbar() {
            self.precedences[SyntaxOp::Or as usize] = 3;
            self.precedences[SyntaxOp::Bol as usize] = 2;
            self.precedences[SyntaxOp::Eol as usize] = 2;
        } else {
            self.precedences[SyntaxOp::Or as usize] = 2;
            self.precedences[SyntaxOp::Bol as usize] = 3;
            self.precedences[SyntaxOp::Eol as usize] = 3;
        }

        self.precedences[SyntaxOp::ClosePar as usize] = 1;
        self.precedences[SyntaxOp::End as usize] = 0;
    }

    /// Get next character from pattern
    fn next_char(&mut self) -> Result<char> {
        if self.pos >= self.pattern.len() {
            Err(RegexError::PrematureEnd)
        } else {
            let ch = self.pattern[self.pos];
            self.pos += 1;
            Ok(ch)
        }
    }

    /// Store a byte in the output buffer
    fn store(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    /// Store an opcode
    fn store_opcode(&mut self, opcode: CompiledOp) {
        self.store(opcode.to_byte());
    }

    /// Store opcode with one byte argument
    fn store_opcode_and_arg(&mut self, opcode: CompiledOp, arg: u8) {
        self.store_opcode(opcode);
        self.store(arg);
    }

    /// Store opcode with one character argument (encoded as UTF-8)
    fn store_opcode_and_char(&mut self, opcode: CompiledOp, ch: char) {
        self.store_opcode(opcode);
        self.store_char(ch);
    }

    /// Store a character as UTF-8 bytes in the bytecode
    fn store_char(&mut self, ch: char) {
        let mut bytes = [0; 4];
        let utf8_bytes = ch.encode_utf8(&mut bytes);
        // First store the length of the UTF-8 encoding
        self.store(utf8_bytes.len() as u8);
        // Then store the UTF-8 bytes
        for &byte in utf8_bytes.as_bytes() {
            self.store(byte);
        }
    }

    /// Get current buffer position for level start tracking
    fn current_level_start(&self) -> usize {
        self.starts[self.starts_base + self.current_level]
    }

    /// Set level start to current position
    fn set_level_start(&mut self) {
        self.starts[self.starts_base + self.current_level] = self.buffer.len();
    }

    /// Push new level starts
    fn push_level_starts(&mut self) -> Result<()> {
        if self.starts_base < (MAX_NESTING - 1) * NUM_LEVELS {
            self.starts_base += NUM_LEVELS;
            Ok(())
        } else {
            Err(RegexError::TooComplex)
        }
    }

    /// Pop level starts
    fn pop_level_starts(&mut self) {
        self.starts_base -= NUM_LEVELS;
    }

    /// Store a 16-bit displacement at given offset
    fn put_addr(&mut self, offset: usize, addr: usize) {
        let disp = (addr as i32) - (offset as i32) - 2;
        self.buffer[offset] = (disp & 0xff) as u8;
        self.buffer[offset + 1] = ((disp >> 8) & 0xff) as u8;
    }

    /// Insert a jump instruction at given position
    fn insert_jump(&mut self, pos: usize, opcode: CompiledOp, addr: usize) {
        // Calculate displacement from after the jump instruction
        let disp = (addr as i32) - (pos as i32) - 3;

        // Insert 3 bytes at position
        self.buffer.insert(pos, ((disp >> 8) & 0xff) as u8); // High byte
        self.buffer.insert(pos, (disp & 0xff) as u8); // Low byte
        self.buffer.insert(pos, opcode.to_byte()); // Opcode

        // Update any stored positions that are after the insertion point
        for i in 0..self.num_jumps {
            if self.future_jumps[i] >= pos {
                self.future_jumps[i] += 3;
            }
        }
    }

    /// Parse hexadecimal escape sequence
    fn get_hex(&mut self) -> Result<char> {
        let ch1 = self.next_char()?;
        let val1 = hex_char_to_decimal(ch1)?;

        let ch2 = self.next_char()?;
        let val2 = hex_char_to_decimal(ch2)?;

        let byte_val = val1 * 16 + val2;
        // Convert byte value to char (only works for ASCII range)
        if byte_val <= 127 {
            Ok(byte_val as char)
        } else {
            Err(RegexError::BadHexEscape)
        }
    }

    /// Translate ANSI escape sequences
    fn ansi_translate(&mut self, ch: char) -> Result<char> {
        let result = match ch {
            'a' | 'A' => '\x07', // audible bell
            'b' | 'B' => '\x08', // backspace
            'f' | 'F' => '\x0C', // form feed
            'n' | 'N' => '\n',   // line feed
            'r' | 'R' => '\r',   // carriage return
            't' | 'T' => '\t',   // tab
            'v' | 'V' => '\x0B', // vertical tab
            'x' | 'X' => return self.get_hex(),
            _ => {
                // Apply translation table if available
                if let Some(ref translate) = self.translate {
                    translate.get(&ch).copied().unwrap_or(ch)
                } else {
                    ch
                }
            }
        };
        Ok(result)
    }

    /// Main compilation loop
    fn compile(mut self) -> Result<Regex> {
        self.set_level_start();
        let mut op = SyntaxOp::Normal;

        // Main parsing loop
        while op != SyntaxOp::End {
            let ch = if self.pos >= self.pattern.len() {
                op = SyntaxOp::End;
                '\0'
            } else {
                let mut ch = self.next_char()?;

                // Apply translation if available
                if let Some(ref translate) = self.translate {
                    ch = translate.get(&ch).copied().unwrap_or(ch);
                }

                op = self.plain_ops.get(&ch).copied().unwrap_or(SyntaxOp::Normal);

                if op == SyntaxOp::Quote {
                    ch = self.next_char()?;
                    op = self
                        .quoted_ops
                        .get(&ch)
                        .copied()
                        .unwrap_or(SyntaxOp::Normal);

                    if op == SyntaxOp::Normal && self.syntax.ansi_sequences() {
                        ch = self.ansi_translate(ch)?;
                    }
                }
                ch
            };

            let level = self.precedences[op as usize];
            self.handle_precedence(level)?;
            self.process_operation(op, ch)?;

            self.beginning_context = matches!(op, SyntaxOp::OpenPar | SyntaxOp::Or);
        }

        // Note: Original C version doesn't check for unmatched parentheses
        // We maintain compatibility by allowing unclosed groups

        // Store end opcode
        self.store_opcode(CompiledOp::End);

        Ok(Regex {
            buffer: self.buffer,
            translate: self.translate,
            syntax: self.syntax,
        })
    }

    /// Handle operator precedence and level management
    fn handle_precedence(&mut self, level: u8) -> Result<()> {
        if level > self.current_level as u8 {
            // Increase precedence level
            while (self.current_level as u8) < level {
                self.current_level += 1;
                self.set_level_start();
            }
        } else if (level as usize) < self.current_level {
            // Decrease precedence level
            self.current_level = level as usize;

            // Update pending jumps
            while self.num_jumps > 0
                && self.future_jumps[self.num_jumps - 1] >= self.current_level_start()
            {
                self.num_jumps -= 1;
                self.put_addr(self.future_jumps[self.num_jumps], self.buffer.len());
            }
        }
        Ok(())
    }

    /// Process individual syntax operations
    fn process_operation(&mut self, op: SyntaxOp, ch: char) -> Result<()> {
        match op {
            SyntaxOp::End => {}

            SyntaxOp::Normal => {
                self.set_level_start();
                self.store_opcode_and_char(CompiledOp::Exact, ch);
            }

            SyntaxOp::AnyChar => {
                self.set_level_start();
                self.store_opcode(CompiledOp::AnyChar);
            }

            SyntaxOp::Bol => {
                if !self.beginning_context && !self.syntax.context_indep_ops() {
                    // Treat as normal character
                    self.set_level_start();
                    self.store_opcode_and_char(CompiledOp::Exact, '^');
                } else if !self.beginning_context {
                    return Err(RegexError::BadSpecialChar);
                } else {
                    self.set_level_start();
                    self.store_opcode(CompiledOp::Bol);
                }
            }

            SyntaxOp::Eol => {
                if !self.is_eol_context() && !self.syntax.context_indep_ops() {
                    // Treat as normal character
                    self.set_level_start();
                    self.store_opcode_and_char(CompiledOp::Exact, '$');
                } else if !self.is_eol_context() {
                    return Err(RegexError::BadSpecialChar);
                } else {
                    self.set_level_start();
                    self.store_opcode(CompiledOp::Eol);
                }
            }

            SyntaxOp::Optional => {
                if self.beginning_context {
                    if self.syntax.context_indep_ops() {
                        return Err(RegexError::BadSpecialChar);
                    } else {
                        self.set_level_start();
                        self.store_opcode_and_char(CompiledOp::Exact, '?');
                        return Ok(());
                    }
                }

                if self.current_level_start() == self.buffer.len() {
                    return Ok(()); // Ignore empty patterns for ?
                }

                self.insert_jump(
                    self.current_level_start(),
                    CompiledOp::FailureJump,
                    self.buffer.len() + 3,
                );
            }

            SyntaxOp::Star => {
                if self.beginning_context {
                    if self.syntax.context_indep_ops() {
                        return Err(RegexError::BadSpecialChar);
                    } else {
                        self.set_level_start();
                        self.store_opcode_and_char(CompiledOp::Exact, '*');
                        return Ok(());
                    }
                }

                if self.current_level_start() == self.buffer.len() {
                    return Ok(()); // Ignore empty patterns
                }

                self.insert_jump(
                    self.current_level_start(),
                    CompiledOp::FailureJump,
                    self.buffer.len() + 6,
                );
                self.insert_jump(
                    self.buffer.len(),
                    CompiledOp::StarJump,
                    self.current_level_start(),
                );
            }

            SyntaxOp::Plus => {
                if self.beginning_context {
                    if self.syntax.context_indep_ops() {
                        return Err(RegexError::BadSpecialChar);
                    } else {
                        self.set_level_start();
                        self.store_opcode_and_char(CompiledOp::Exact, '+');
                        return Ok(());
                    }
                }

                if self.current_level_start() == self.buffer.len() {
                    return Ok(()); // Ignore empty patterns
                }

                // Follow the original regexpr.c algorithm exactly:
                // 1. Insert failure_jump at start
                // 2. Insert star_jump at end
                // 3. For plus, insert dummy_failure_jump at very start
                self.insert_jump(
                    self.current_level_start(),
                    CompiledOp::FailureJump,
                    self.buffer.len() + 6,
                );
                self.insert_jump(
                    self.buffer.len(),
                    CompiledOp::StarJump,
                    self.current_level_start(),
                );

                // Plus-specific: insert dummy_failure_jump to skip the first failure_jump
                self.insert_jump(
                    self.current_level_start(),
                    CompiledOp::DummyFailureJump,
                    self.current_level_start() + 6,
                );
            }

            SyntaxOp::Or => {
                self.insert_jump(
                    self.current_level_start(),
                    CompiledOp::FailureJump,
                    self.buffer.len() + 6,
                );

                if self.num_jumps >= MAX_NESTING {
                    return Err(RegexError::TooComplex);
                }

                self.store_opcode(CompiledOp::Jump);
                self.future_jumps[self.num_jumps] = self.buffer.len();
                self.num_jumps += 1;
                self.store(0);
                self.store(0);
                self.set_level_start();
            }

            SyntaxOp::OpenPar => {
                self.set_level_start();

                if self.next_register < RE_NREGS as u8 {
                    self.store_opcode_and_arg(CompiledOp::StartMemory, self.next_register);
                    self.open_registers[self.num_open_registers] = self.next_register;
                    self.num_open_registers += 1;
                    self.next_register += 1;
                }

                self.paren_depth += 1;
                self.push_level_starts()?;
                self.current_level = 0;
                self.set_level_start();
            }

            SyntaxOp::ClosePar => {
                // Note: Original C version allows unmatched closing parentheses
                // We maintain compatibility by not checking paren_depth == 0

                if self.paren_depth > 0 {
                    self.pop_level_starts();
                    self.current_level = self.precedences[SyntaxOp::OpenPar as usize] as usize;
                    self.paren_depth -= 1;

                    if self.paren_depth < self.num_open_registers {
                        self.num_open_registers -= 1;
                        self.store_opcode_and_arg(
                            CompiledOp::EndMemory,
                            self.open_registers[self.num_open_registers],
                        );
                    }
                } else {
                    // Treat as normal character when no matching open paren
                    self.set_level_start();
                    self.store_opcode_and_char(CompiledOp::Exact, ')');
                }
            }

            SyntaxOp::Memory => {
                if ch == '0' {
                    return Err(RegexError::BadBackReference);
                }

                let reg_num = (ch as u8) - b'0';
                self.set_level_start();
                self.store_opcode_and_arg(CompiledOp::MatchMemory, reg_num);
            }

            SyntaxOp::OpenSet => {
                // Character set implementation
                self.compile_character_set()?;
            }

            SyntaxOp::WordBound => {
                self.set_level_start();
                self.store_opcode(CompiledOp::WordBound);
            }

            SyntaxOp::NotWordBound => {
                self.set_level_start();
                self.store_opcode(CompiledOp::NotWordBound);
            }

            SyntaxOp::WordChar => {
                self.set_level_start();
                self.store_opcode_and_arg(CompiledOp::SyntaxSpec, 1); // Sword = 1
            }

            SyntaxOp::NotWordChar => {
                self.set_level_start();
                self.store_opcode_and_arg(CompiledOp::NotSyntaxSpec, 1); // Sword = 1
            }

            SyntaxOp::WordBeg => {
                self.set_level_start();
                self.store_opcode(CompiledOp::WordBeg);
            }

            SyntaxOp::WordEnd => {
                self.set_level_start();
                self.store_opcode(CompiledOp::WordEnd);
            }

            SyntaxOp::BegBuf => {
                self.set_level_start();
                self.store_opcode(CompiledOp::BegBuf);
            }

            SyntaxOp::EndBuf => {
                self.set_level_start();
                self.store_opcode(CompiledOp::EndBuf);
            }

            SyntaxOp::ExtendedMemory => {
                // \vNN for registers 10-99
                let ch1 = self.next_char()?;
                let ch2 = self.next_char()?;

                if !ch1.is_ascii_digit() || !ch2.is_ascii_digit() {
                    return Err(RegexError::BadBackReference);
                }

                let reg_num = ((ch1 as u8) - b'0') * 10 + ((ch2 as u8) - b'0');
                if reg_num == 0 || reg_num >= RE_NREGS as u8 {
                    return Err(RegexError::BadBackReference);
                }

                self.set_level_start();
                self.store_opcode_and_arg(CompiledOp::MatchMemory, reg_num);
            }

            _ => {
                return Err(RegexError::CompileError(format!(
                    "Unimplemented operation: {op:?}"
                )));
            }
        }

        Ok(())
    }

    /// Check if we're in a context where $ can appear
    fn is_eol_context(&self) -> bool {
        // This is a simplified version - the original has more complex logic
        self.pos >= self.pattern.len()
            || self.pattern[self.pos] == '|'
            || self.pattern[self.pos] == ')'
    }

    /// Compile character set [abc] or [^abc] or [a-z]
    fn compile_character_set(&mut self) -> Result<()> {
        self.set_level_start();
        self.store_opcode(CompiledOp::Set);

        // For Unicode support, we'll store character ranges instead of a bitset
        // Format: [complement_flag][num_ranges][range1_start][range1_end]...[rangeN_start][rangeN_end]

        // Check for negation
        let mut complement = false;
        if self.pos < self.pattern.len() && self.pattern[self.pos] == '^' {
            complement = true;
            self.pos += 1;
        }

        // Store complement flag
        self.store(if complement { 1 } else { 0 });

        // Reserve space for number of ranges (will be filled in later)
        let num_ranges_pos = self.buffer.len();
        self.store(0);

        let mut ranges: Vec<(char, char)> = Vec::new();
        let mut prev_char: Option<char> = None;
        let mut in_range = false;
        let mut first_char = true;

        let mut found_closing = false;
        while self.pos < self.pattern.len() {
            let ch = self.pattern[self.pos];
            self.pos += 1;

            // Handle closing bracket
            if ch == ']' && !first_char {
                found_closing = true;
                break;
            }
            first_char = false;

            let mut actual_char = ch;

            // Handle ANSI escape sequences if enabled
            if ch == '\\' && self.syntax.ansi_sequences() && self.pos < self.pattern.len() {
                let escaped = self.pattern[self.pos];
                self.pos += 1;
                actual_char = self.ansi_translate(escaped)?;
            }

            // Apply translation if available
            if let Some(ref translate) = self.translate {
                actual_char = translate.get(&actual_char).copied().unwrap_or(actual_char);
            }

            if in_range {
                // We're completing a range like a-z
                if let Some(start_char) = prev_char {
                    ranges.push((start_char, actual_char));
                }
                in_range = false;
                prev_char = None;
            } else if ch == '-'
                && prev_char.is_some()
                && self.pos < self.pattern.len()
                && self.pattern[self.pos] != ']'
            {
                // Start of range
                in_range = true;
            } else {
                // Regular character - treat as single char range
                ranges.push((actual_char, actual_char));
                prev_char = Some(actual_char);
            }
        }

        // Check if closing bracket was found
        if !found_closing {
            return Err(RegexError::PrematureEnd);
        }

        // Handle trailing dash
        if in_range {
            ranges.push(('-', '-'));
        }

        // Store number of ranges
        self.buffer[num_ranges_pos] = ranges.len() as u8;

        // Store all ranges
        for (start, end) in ranges {
            self.store_char(start);
            self.store_char(end);
        }

        Ok(())
    }
}

/// Convert hexadecimal character to decimal
fn hex_char_to_decimal(ch: char) -> Result<u8> {
    match ch {
        '0'..='9' => Ok((ch as u8) - b'0'),
        'a'..='f' => Ok((ch as u8) - b'a' + 10),
        'A'..='F' => Ok((ch as u8) - b'A' + 10),
        _ => Err(RegexError::BadHexEscape),
    }
}

/// Main compilation entry point
pub fn compile(pattern: &str, syntax: SyntaxFlags) -> Result<Regex> {
    let compiler = Compiler::new(pattern, syntax)?;
    compiler.compile()
}
