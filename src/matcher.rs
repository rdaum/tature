//! Regex pattern matching engine with tick-based execution limits
//!
//! This module ports the matching logic from re_match_2 and re_search_2
//! in regexpr.c:880-1464

use crate::{
    error::{RegexError, Result},
    opcodes::CompiledOp,
    Captures, ExecLimits, Regex, RE_NREGS,
};

/// Initial size of failure stack
const INITIAL_FAILURES: usize = 128;

/// A failure point for backtracking
#[derive(Debug, Clone)]
struct FailurePoint {
    /// Position in text when failure occurred
    text_pos: usize,
    /// Position in bytecode to resume from
    code_pos: usize,
}

/// Execution state for the regex virtual machine
struct MatchState<'a> {
    /// The compiled regex
    regex: &'a Regex,
    /// Input text as chars
    text_chars: Vec<char>,
    /// Current position in text (char index)
    text_pos: usize,
    /// Position in bytecode
    code_pos: usize,
    /// Failure stack for backtracking
    failure_stack: Vec<FailurePoint>,
    /// Capture group start positions
    reg_start_pos: [Option<usize>; RE_NREGS],
    /// Capture group end positions
    reg_end_pos: [Option<usize>; RE_NREGS],
    /// Temporary capture group positions
    reg_maybe_pos: [Option<usize>; RE_NREGS],
    /// Execution limits
    limits: ExecLimits,
    /// Current tick count
    ticks: usize,
}

impl<'a> MatchState<'a> {
    /// Create new match state
    fn new(regex: &'a Regex, text: &str, limits: ExecLimits) -> Self {
        Self {
            regex,
            text_chars: text.chars().collect(),
            text_pos: 0,
            code_pos: 0,
            failure_stack: Vec::with_capacity(INITIAL_FAILURES),
            reg_start_pos: [None; RE_NREGS],
            reg_end_pos: [None; RE_NREGS],
            reg_maybe_pos: [None; RE_NREGS],
            limits,
            ticks: 0,
        }
    }

    /// Check if we've exceeded execution limits
    fn check_limits(&mut self) -> Result<()> {
        self.ticks += 1;

        if let Some(max_ticks) = self.limits.max_ticks {
            if self.ticks >= max_ticks {
                return Err(RegexError::Timeout);
            }
        }

        if self.failure_stack.len() >= self.limits.max_failures {
            return Err(RegexError::ExecutionError);
        }

        Ok(())
    }

    /// Get current character and advance position
    fn next_char(&mut self) -> Result<char> {
        let ch = self.current_char()?;
        self.advance();
        Ok(ch)
    }

    /// Get current character without advancing
    fn current_char(&self) -> Result<char> {
        if self.text_pos >= self.text_chars.len() {
            return Err(RegexError::ExecutionError);
        }

        let ch = self.text_chars[self.text_pos];

        // Apply translation if available
        if let Some(ref translate) = self.regex.translate {
            Ok(translate.get(&ch).copied().unwrap_or(ch))
        } else {
            Ok(ch)
        }
    }

    /// Advance text position
    fn advance(&mut self) {
        self.text_pos += 1;
    }

    /// Check if we're at end of text
    fn at_end(&self) -> bool {
        self.text_pos >= self.text_chars.len()
    }

    /// Get absolute position in text
    fn absolute_pos(&self) -> usize {
        self.text_pos
    }

    /// Push failure point onto stack
    fn push_failure(&mut self, code_pos: usize) -> Result<()> {
        if self.failure_stack.len() >= self.limits.max_failures {
            return Err(RegexError::ExecutionError);
        }

        self.failure_stack.push(FailurePoint {
            text_pos: self.text_pos,
            code_pos,
        });
        Ok(())
    }

    /// Pop failure point and backtrack
    fn pop_failure(&mut self) -> bool {
        if let Some(failure) = self.failure_stack.pop() {
            self.text_pos = failure.text_pos;
            self.code_pos = failure.code_pos;
            true
        } else {
            false
        }
    }

    /// Read 16-bit displacement from bytecode
    fn read_displacement(&mut self) -> i16 {
        let low = self.regex.buffer[self.code_pos] as i16;
        let high = self.regex.buffer[self.code_pos + 1] as i16;
        self.code_pos += 2;
        low | (high << 8)
    }

    /// Main matching loop
    fn execute(&mut self, start_pos: usize) -> Result<Option<usize>> {
        // Set up initial position
        self.text_pos = start_pos;
        self.code_pos = 0;

        loop {
            self.check_limits()?;

            if self.code_pos >= self.regex.buffer.len() {
                return Err(RegexError::ExecutionError);
            }

            let opcode = CompiledOp::from_byte(self.regex.buffer[self.code_pos])
                .ok_or(RegexError::ExecutionError)?;
            self.code_pos += 1;

            match opcode {
                CompiledOp::End => {
                    // Match successful
                    return Ok(Some(self.absolute_pos()));
                }

                CompiledOp::Bol => {
                    // Beginning of line
                    if self.text_pos == 0 {
                        // At very beginning
                        continue;
                    }

                    // Check if previous character was newline
                    let prev_char = if self.text_pos > 0 {
                        Some(self.text_chars[self.text_pos - 1])
                    } else {
                        None
                    };

                    if prev_char == Some('\n') {
                        continue;
                    }

                    self.backtrack()?;
                }

                CompiledOp::Eol => {
                    // End of line
                    if self.at_end() {
                        continue;
                    }

                    if self.current_char()? == '\n' {
                        continue;
                    }

                    self.backtrack()?;
                }

                CompiledOp::Set => {
                    // Character set with Unicode ranges
                    match self.next_char() {
                        Ok(ch) => {
                            // Read complement flag
                            let complement = self.regex.buffer[self.code_pos] != 0;
                            self.code_pos += 1;

                            // Read number of ranges
                            let num_ranges = self.regex.buffer[self.code_pos] as usize;
                            self.code_pos += 1;

                            let mut matched = false;
                            let mut pos = self.code_pos;

                            // Check each range
                            for _ in 0..num_ranges {
                                // Read start char
                                let start_len = self.regex.buffer[pos] as usize;
                                pos += 1;
                                let start_bytes = &self.regex.buffer[pos..pos + start_len];
                                let start_char = std::str::from_utf8(start_bytes)
                                    .unwrap()
                                    .chars()
                                    .next()
                                    .unwrap();
                                pos += start_len;

                                // Read end char
                                let end_len = self.regex.buffer[pos] as usize;
                                pos += 1;
                                let end_bytes = &self.regex.buffer[pos..pos + end_len];
                                let end_char = std::str::from_utf8(end_bytes)
                                    .unwrap()
                                    .chars()
                                    .next()
                                    .unwrap();
                                pos += end_len;

                                // Check if character is in this range (but don't break early)
                                if ch >= start_char && ch <= end_char {
                                    matched = true;
                                    // Don't break - continue reading all ranges to advance pos correctly
                                }
                            }

                            // Apply complement if needed
                            if complement {
                                matched = !matched;
                            }

                            if matched {
                                self.code_pos = pos; // Skip to end of character set data
                                continue;
                            }

                            self.backtrack()?;
                        }
                        Err(_) => {
                            // No more characters available - backtrack
                            self.backtrack()?;
                        }
                    }
                }

                CompiledOp::Exact => {
                    // Exact character match - read UTF-8 encoded char from bytecode
                    let char_len = self.regex.buffer[self.code_pos] as usize;
                    self.code_pos += 1;
                    let char_bytes = &self.regex.buffer[self.code_pos..self.code_pos + char_len];
                    let expected = std::str::from_utf8(char_bytes)
                        .unwrap()
                        .chars()
                        .next()
                        .unwrap();
                    self.code_pos += char_len;

                    match self.next_char() {
                        Ok(ch) => {
                            if ch == expected {
                                continue;
                            }
                            self.backtrack()?;
                        }
                        Err(_) => {
                            // No more characters available - backtrack
                            self.backtrack()?;
                        }
                    }
                }

                CompiledOp::AnyChar => {
                    // Any character except newline
                    match self.next_char() {
                        Ok(ch) => {
                            if ch != '\n' {
                                continue;
                            }
                            self.backtrack()?;
                        }
                        Err(_) => {
                            // No more characters available - backtrack
                            self.backtrack()?;
                        }
                    }
                }

                CompiledOp::StartMemory => {
                    // Start capture group
                    let reg = self.regex.buffer[self.code_pos] as usize;
                    self.code_pos += 1;

                    if reg < RE_NREGS {
                        self.reg_maybe_pos[reg] = Some(self.absolute_pos());
                    }
                }

                CompiledOp::EndMemory => {
                    // End capture group
                    let reg = self.regex.buffer[self.code_pos] as usize;
                    self.code_pos += 1;

                    if reg < RE_NREGS {
                        self.reg_start_pos[reg] = self.reg_maybe_pos[reg];
                        self.reg_end_pos[reg] = Some(self.absolute_pos());
                    }
                }

                CompiledOp::MatchMemory => {
                    // Match previous capture group
                    let reg = self.regex.buffer[self.code_pos] as usize;
                    self.code_pos += 1;

                    if reg >= RE_NREGS || self.reg_end_pos[reg].is_none() {
                        self.backtrack()?;
                        continue;
                    }

                    // This would need implementation to match the captured text
                    // For now, simplified version
                    self.backtrack()?;
                }

                CompiledOp::Jump => {
                    // Unconditional jump
                    let disp = self.read_displacement();
                    self.code_pos = (self.code_pos as i32 + disp as i32) as usize;
                }

                CompiledOp::DummyFailureJump => {
                    // DummyFailureJump is used in plus quantifiers
                    let _disp = self.read_displacement();

                    // The next instruction should be a FailureJump
                    if self.code_pos < self.regex.buffer.len()
                        && CompiledOp::from_byte(self.regex.buffer[self.code_pos])
                            == Some(CompiledOp::FailureJump)
                    {
                        // Skip the FailureJump instruction entirely on first iteration
                        // This forces the pattern to match at least once before allowing failure
                        self.code_pos += 3; // Skip FailureJump opcode + 2 displacement bytes
                    } else {
                        // Should not happen in well-formed bytecode
                        return Err(RegexError::ExecutionError);
                    }
                }

                CompiledOp::FailureJump => {
                    // Regular failure jump
                    if self.failure_stack.len() >= self.limits.max_failures {
                        return Err(RegexError::ExecutionError);
                    }

                    let disp = self.read_displacement();
                    let target = (self.code_pos as i32 + disp as i32) as usize;
                    self.push_failure(target)?;
                }

                CompiledOp::StarJump => {
                    // Star jump - this should be converted to UpdateFailureJump during execution
                    let disp = self.read_displacement();
                    let target = (self.code_pos as i32 + disp as i32) as usize;

                    // For now, treat as UpdateFailureJump
                    if !self.failure_stack.is_empty() {
                        let last_idx = self.failure_stack.len() - 1;
                        self.failure_stack[last_idx].text_pos = self.text_pos;
                    }

                    self.code_pos = target;
                }

                CompiledOp::UpdateFailureJump => {
                    // Update failure point and jump
                    if !self.failure_stack.is_empty() {
                        let last_idx = self.failure_stack.len() - 1;
                        self.failure_stack[last_idx].text_pos = self.text_pos;
                    }

                    let disp = self.read_displacement();
                    self.code_pos = (self.code_pos as i32 + disp as i32) as usize;
                }

                CompiledOp::WordBeg => {
                    // Beginning of word (\<)
                    // Must be at word boundary where next char is word char and prev is not
                    if self.at_end() {
                        self.backtrack()?;
                        continue;
                    }

                    let next_char = self.current_char()?;
                    let next_is_word = is_word_char(next_char);

                    if !next_is_word {
                        self.backtrack()?;
                        continue;
                    }

                    // Check previous character
                    let prev_is_word = if self.absolute_pos() == 0 {
                        false // Beginning of text
                    } else {
                        is_word_char(self.text_chars[self.text_pos - 1])
                    };

                    if prev_is_word {
                        self.backtrack()?;
                    }
                }

                CompiledOp::WordEnd => {
                    // End of word (\>)
                    // Must be at word boundary where prev char is word char and next is not
                    if self.absolute_pos() == 0 {
                        self.backtrack()?;
                        continue;
                    }

                    // Check previous character
                    let prev_is_word = if self.absolute_pos() == 0 {
                        false
                    } else {
                        is_word_char(self.text_chars[self.text_pos - 1])
                    };

                    if !prev_is_word {
                        self.backtrack()?;
                        continue;
                    }

                    // Check next character
                    let next_is_word = if self.at_end() {
                        false // End of text
                    } else {
                        let next_char = self.current_char()?;
                        is_word_char(next_char)
                    };

                    if next_is_word {
                        self.backtrack()?;
                    }
                }

                CompiledOp::WordBound => {
                    // Word boundary (\b)
                    // Match at boundary between word and non-word characters
                    let at_start = self.absolute_pos() == 0;
                    let at_end = self.at_end();

                    if at_start || at_end {
                        // At text boundaries, check if adjacent char is word char
                        if at_start && !at_end {
                            let next_char = self.current_char()?;
                            if !is_word_char(next_char) {
                                self.backtrack()?;
                            }
                        } else if at_end && !at_start {
                            let prev_is_word = if self.text_pos > 0 {
                                is_word_char(self.text_chars[self.text_pos - 1])
                            } else {
                                false
                            };
                            if !prev_is_word {
                                self.backtrack()?;
                            }
                        }
                        // At very start or very end is always a word boundary
                    } else {
                        // In middle of text - check both sides
                        let prev_is_word = if self.text_pos > 0 {
                            is_word_char(self.text_chars[self.text_pos - 1])
                        } else {
                            false
                        };

                        let next_char = self.current_char()?;
                        let next_is_word = is_word_char(next_char);

                        // Word boundary exists when prev and next have different word-ness
                        if prev_is_word == next_is_word {
                            self.backtrack()?;
                        }
                    }
                }

                CompiledOp::NotWordBound => {
                    // Not word boundary (\B)
                    // Match when NOT at boundary between word and non-word characters
                    let at_start = self.absolute_pos() == 0;
                    let at_end = self.at_end();

                    if at_start || at_end {
                        // At text boundaries, this never matches
                        self.backtrack()?;
                    } else {
                        // In middle of text - check both sides
                        let prev_is_word = if self.text_pos > 0 {
                            is_word_char(self.text_chars[self.text_pos - 1])
                        } else {
                            false
                        };

                        let next_char = self.current_char()?;
                        let next_is_word = is_word_char(next_char);

                        // Not word boundary when prev and next have same word-ness
                        if prev_is_word != next_is_word {
                            self.backtrack()?;
                        }
                    }
                }

                CompiledOp::SyntaxSpec => {
                    // Match character with specific syntax (\w)
                    let syntax_code = self.regex.buffer[self.code_pos];
                    self.code_pos += 1;

                    match self.next_char() {
                        Ok(ch) => {
                            // For now, only handle Sword (1) for word characters
                            if syntax_code == 1 && is_word_char(ch) {
                                continue;
                            }

                            self.backtrack()?;
                        }
                        Err(_) => {
                            // No more characters available - backtrack
                            self.backtrack()?;
                        }
                    }
                }

                CompiledOp::NotSyntaxSpec => {
                    // Match character without specific syntax (\W)
                    let syntax_code = self.regex.buffer[self.code_pos];
                    self.code_pos += 1;

                    match self.next_char() {
                        Ok(ch) => {
                            // For now, only handle Sword (1) for word characters
                            if syntax_code == 1 && !is_word_char(ch) {
                                continue;
                            }

                            self.backtrack()?;
                        }
                        Err(_) => {
                            // No more characters available - backtrack
                            self.backtrack()?;
                        }
                    }
                }

                _ => {
                    // Other opcodes would be implemented here
                    return Err(RegexError::ExecutionError);
                }
            }
        }
    }

    /// Backtrack on failure
    fn backtrack(&mut self) -> Result<()> {
        if !self.pop_failure() {
            Err(RegexError::ExecutionError)
        } else {
            Ok(())
        }
    }

    /// Build captures result
    fn build_captures(&self, match_start: usize, match_end: usize) -> Captures {
        let mut captures = Captures {
            groups: [(None, None); RE_NREGS],
        };

        // Set match group 0
        captures.groups[0] = (Some(match_start), Some(match_end));

        // Set other capture groups
        for i in 1..RE_NREGS {
            if let (Some(start), Some(end)) = (self.reg_start_pos[i], self.reg_end_pos[i]) {
                captures.groups[i] = (Some(start), Some(end));
            }
        }

        captures
    }
}

/// Check if character is a word character
fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

/// Search for pattern in text
pub fn search(
    regex: &Regex,
    text: &str,
    start: usize,
    range: i32,
    limits: ExecLimits,
) -> Result<i32> {
    let text_len = text.chars().count();
    let end = if range >= 0 {
        std::cmp::min(start + range as usize, text_len)
    } else {
        start.saturating_sub((-range) as usize)
    };

    if range >= 0 {
        // Forward search
        for pos in start..=end {
            if pos > text_len {
                break;
            }
            let mut state = MatchState::new(regex, text, limits);
            if let Ok(Some(_)) = state.execute(pos) {
                return Ok(pos as i32);
            }
        }
    } else {
        // Backward search
        for pos in (end..=start).rev() {
            if pos > text_len {
                continue;
            }
            let mut state = MatchState::new(regex, text, limits);
            if let Ok(Some(_)) = state.execute(pos) {
                return Ok(pos as i32);
            }
        }
    }

    Ok(-1)
}

/// Match pattern at specific position
pub fn match_at(
    regex: &Regex,
    text: &str,
    pos: usize,
    limits: ExecLimits,
) -> Result<Option<Captures>> {
    let mut state = MatchState::new(regex, text, limits);

    if let Ok(Some(end_pos)) = state.execute(pos) {
        Ok(Some(state.build_captures(pos, end_pos)))
    } else {
        Ok(None)
    }
}
