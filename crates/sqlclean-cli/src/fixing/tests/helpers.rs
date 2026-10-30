use crate::checking::models::{Finding, FixSafety};
use crate::positions::models::Edit;

/// A finding in `q.sql` whose fix applies `edits`.
pub(crate) fn finding(
    code: &str,
    fix: Option<FixSafety>,
    edits: &[(usize, usize, &str)],
) -> Finding {
    Finding {
        code: code.to_owned(),
        path: "q.sql".to_owned(),
        start: edits.first().map_or(0, |edit| edit.0),
        end: 0,
        message: String::new(),
        remediation: String::new(),
        edits: edits
            .iter()
            .map(|(start, end, text)| Edit {
                start: *start,
                end: *end,
                text: (*text).to_owned(),
            })
            .collect(),
        fix,
    }
}
