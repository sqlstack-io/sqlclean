use crate::checking::models::CheckOutcome;
use crate::discovery::models::SourceFile;

/// The fixed and formatted files, and what remains to fix by hand.
#[derive(Debug, Clone)]
pub struct FixOutcome {
    /// Every input file with its new text; unchanged files keep their text.
    pub files: Vec<SourceFile>,
    pub applied_fixes: usize,
    pub passes: usize,
    /// Files the formatter refused, with its reason.
    pub format_refusals: Vec<(String, String)>,
    pub remaining: CheckOutcome,
}
