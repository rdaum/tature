//! Regex syntax configuration flags

/// Syntax flags that control regex compilation behavior
/// These correspond to the RE_* flags in regexpr.h:42-49
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxFlags(u32);

impl SyntaxFlags {
    /// No quoting needed for parentheses - ( and ) are special
    pub const NO_BK_PARENS: Self = Self(1);
    /// No quoting needed for vertical bar - | is special
    pub const NO_BK_VBAR: Self = Self(2);
    /// Quoting needed for + and ? - \+ and \? are special
    pub const BK_PLUS_QM: Self = Self(4);
    /// | binds tighter than ^ and $
    pub const TIGHT_VBAR: Self = Self(8);
    /// Treat newline as alternation operator
    pub const NEWLINE_OR: Self = Self(16);
    /// ^$?*+ are special in all contexts
    pub const CONTEXT_INDEP_OPS: Self = Self(32);
    /// Enable ANSI sequences (\n, \t, etc) and \xhh
    pub const ANSI_HEX: Self = Self(64);
    /// Disable GNU extensions
    pub const NO_GNU_EXTENSIONS: Self = Self(128);
    /// Case insensitive matching
    pub const CASE_INSENSITIVE: Self = Self(256);

    /// Create empty flags (no bits set)
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Check if this flag set contains the given flag
    pub const fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }

    /// Get the raw bits value
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Create from raw bits, truncating invalid bits
    pub const fn from_bits_truncate(bits: u32) -> Self {
        Self(bits)
    }
}

impl Default for SyntaxFlags {
    fn default() -> Self {
        // Default to EMACS style (no flags set)
        SyntaxFlags::empty()
    }
}

impl SyntaxFlags {
    /// AWK-style regex syntax
    pub const AWK: Self = Self::from_bits_truncate(
        Self::NO_BK_PARENS.bits() | Self::NO_BK_VBAR.bits() | Self::CONTEXT_INDEP_OPS.bits(),
    );

    /// EGREP-style regex syntax  
    pub const EGREP: Self = Self::from_bits_truncate(Self::AWK.bits() | Self::NEWLINE_OR.bits());

    /// GREP-style regex syntax
    pub const GREP: Self =
        Self::from_bits_truncate(Self::BK_PLUS_QM.bits() | Self::NEWLINE_OR.bits());

    /// EMACS-style regex syntax (default)
    pub const EMACS: Self = Self::empty();

    /// LambdaMOO-style regex syntax (context independent operations)
    pub const MOO: Self = Self::CONTEXT_INDEP_OPS;

    /// Check if parentheses need backslash quoting
    pub fn needs_backslash_parens(self) -> bool {
        !self.contains(Self::NO_BK_PARENS)
    }

    /// Check if vertical bar needs backslash quoting
    pub fn needs_backslash_vbar(self) -> bool {
        !self.contains(Self::NO_BK_VBAR)
    }

    /// Check if plus and question mark need backslash quoting
    pub fn needs_backslash_plus_qm(self) -> bool {
        self.contains(Self::BK_PLUS_QM)
    }

    /// Check if vertical bar binds tighter than ^ and $
    pub fn tight_vbar(self) -> bool {
        self.contains(Self::TIGHT_VBAR)
    }

    /// Check if newline should be treated as alternation
    pub fn newline_or(self) -> bool {
        self.contains(Self::NEWLINE_OR)
    }

    /// Check if operators are context independent
    pub fn context_indep_ops(self) -> bool {
        self.contains(Self::CONTEXT_INDEP_OPS)
    }

    /// Check if ANSI escape sequences are enabled
    pub fn ansi_sequences(self) -> bool {
        self.contains(Self::ANSI_HEX)
    }

    /// Check if GNU extensions are disabled
    pub fn no_gnu_extensions(self) -> bool {
        self.contains(Self::NO_GNU_EXTENSIONS)
    }

    /// Check if case insensitive matching is enabled
    pub fn case_insensitive(self) -> bool {
        self.contains(Self::CASE_INSENSITIVE)
    }
}
