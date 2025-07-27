//! Regex syntax configuration flags

use bitflags::bitflags;

bitflags! {
    /// Syntax flags that control regex compilation behavior
    /// These correspond to the RE_* flags in regexpr.h:42-49
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SyntaxFlags: u32 {
        /// No quoting needed for parentheses - ( and ) are special
        const NO_BK_PARENS = 1;
        /// No quoting needed for vertical bar - | is special
        const NO_BK_VBAR = 2;
        /// Quoting needed for + and ? - \+ and \? are special
        const BK_PLUS_QM = 4;
        /// | binds tighter than ^ and $
        const TIGHT_VBAR = 8;
        /// Treat newline as alternation operator
        const NEWLINE_OR = 16;
        /// ^$?*+ are special in all contexts
        const CONTEXT_INDEP_OPS = 32;
        /// Enable ANSI sequences (\n, \t, etc) and \xhh
        const ANSI_HEX = 64;
        /// Disable GNU extensions
        const NO_GNU_EXTENSIONS = 128;
        /// Case insensitive matching
        const CASE_INSENSITIVE = 256;
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
