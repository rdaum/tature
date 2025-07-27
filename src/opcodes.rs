//! Bytecode opcodes for the compiled regex virtual machine

/// Compiled regex opcodes (from regexpr.c:41-66)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompiledOp {
    /// End of pattern reached
    End = 0,
    /// Beginning of line
    Bol = 1,
    /// End of line  
    Eol = 2,
    /// Character set (followed by 32 bytes of set)
    Set = 3,
    /// Exact character match (followed by byte to match)
    Exact = 4,
    /// Matches any character except newline
    AnyChar = 5,
    /// Set register start address (followed by register number)
    StartMemory = 6,
    /// Set register end address (followed by register number)
    EndMemory = 7,
    /// Match duplicate of register contents (register number follows)
    MatchMemory = 8,
    /// Jump (followed by two bytes: lsb, msb of displacement)
    Jump = 9,
    /// Will change to jump/update_failure_jump at runtime
    StarJump = 10,
    /// Jump to address on failure
    FailureJump = 11,
    /// Update topmost failure point and jump
    UpdateFailureJump = 12,
    /// Push dummy failure point and jump
    DummyFailureJump = 13,
    /// Match at beginning of buffer
    BegBuf = 14,
    /// Match at end of buffer
    EndBuf = 15,
    /// Match at beginning of word
    WordBeg = 16,
    /// Match at end of word
    WordEnd = 17,
    /// Match if at word boundary
    WordBound = 18,
    /// Match if not at word boundary
    NotWordBound = 19,
    /// Matches syntax code (1 byte follows)
    SyntaxSpec = 20,
    /// Matches if syntax code does not match (1 byte follows)
    NotSyntaxSpec = 21,
}

impl CompiledOp {
    /// Convert byte to opcode
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(CompiledOp::End),
            1 => Some(CompiledOp::Bol),
            2 => Some(CompiledOp::Eol),
            3 => Some(CompiledOp::Set),
            4 => Some(CompiledOp::Exact),
            5 => Some(CompiledOp::AnyChar),
            6 => Some(CompiledOp::StartMemory),
            7 => Some(CompiledOp::EndMemory),
            8 => Some(CompiledOp::MatchMemory),
            9 => Some(CompiledOp::Jump),
            10 => Some(CompiledOp::StarJump),
            11 => Some(CompiledOp::FailureJump),
            12 => Some(CompiledOp::UpdateFailureJump),
            13 => Some(CompiledOp::DummyFailureJump),
            14 => Some(CompiledOp::BegBuf),
            15 => Some(CompiledOp::EndBuf),
            16 => Some(CompiledOp::WordBeg),
            17 => Some(CompiledOp::WordEnd),
            18 => Some(CompiledOp::WordBound),
            19 => Some(CompiledOp::NotWordBound),
            20 => Some(CompiledOp::SyntaxSpec),
            21 => Some(CompiledOp::NotSyntaxSpec),
            _ => None,
        }
    }

    /// Convert opcode to byte
    pub fn to_byte(self) -> u8 {
        self as u8
    }

    /// Get the number of argument bytes this opcode expects
    pub fn arg_count(self) -> usize {
        match self {
            CompiledOp::End
            | CompiledOp::Bol
            | CompiledOp::Eol
            | CompiledOp::AnyChar
            | CompiledOp::BegBuf
            | CompiledOp::EndBuf
            | CompiledOp::WordBeg
            | CompiledOp::WordEnd
            | CompiledOp::WordBound
            | CompiledOp::NotWordBound => 0,

            CompiledOp::Exact
            | CompiledOp::StartMemory
            | CompiledOp::EndMemory
            | CompiledOp::MatchMemory
            | CompiledOp::SyntaxSpec
            | CompiledOp::NotSyntaxSpec => 1,

            CompiledOp::Jump
            | CompiledOp::StarJump
            | CompiledOp::FailureJump
            | CompiledOp::UpdateFailureJump
            | CompiledOp::DummyFailureJump => 2,

            CompiledOp::Set => 0, // Variable length - depends on number of ranges
        }
    }
}

/// Parse syntax operations (from regexpr.c:69-99)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SyntaxOp {
    /// Special code for end of regexp
    End = 0,
    /// Normal character
    Normal = 1,
    /// Any character except newline (.)
    AnyChar = 2,
    /// The quote character (\)
    Quote = 3,
    /// Match beginning of line (^)
    Bol = 4,
    /// Match end of line ($)
    Eol = 5,
    /// Match preceding expression optionally (?)
    Optional = 6,
    /// Match preceding expr zero or more times (*)
    Star = 7,
    /// Match preceding expr one or more times (+)
    Plus = 8,
    /// Match either of alternatives (|)
    Or = 9,
    /// Opening parenthesis
    OpenPar = 10,
    /// Closing parenthesis
    ClosePar = 11,
    /// Match memory register (\1, \2, etc)
    Memory = 12,
    /// Extended memory (\v10-\v99)
    ExtendedMemory = 13,
    /// Open character set ([)
    OpenSet = 14,
    /// Beginning of buffer (\`)
    BegBuf = 15,
    /// End of buffer (\')
    EndBuf = 16,
    /// Word character (\w)
    WordChar = 17,
    /// Not word character (\W)
    NotWordChar = 18,
    /// Beginning of word (\<)
    WordBeg = 19,
    /// End of word (\>)
    WordEnd = 20,
    /// Word boundary (\b)
    WordBound = 21,
    /// Not word boundary (\B)
    NotWordBound = 22,
}

/// Syntax table entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxType {
    /// Regular character (no special syntax meaning)
    Normal = 0,
    /// Word character (letter, digit, underscore)
    Word = 1,
}
