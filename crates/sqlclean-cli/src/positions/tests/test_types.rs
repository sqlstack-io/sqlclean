pub(crate) struct LineColumnTestCase {
    pub(crate) description: &'static str,
    pub(crate) offset: usize,
    pub(crate) expected_line_column: (usize, usize),
}

pub(crate) struct ApplyEditsTestCase {
    pub(crate) description: &'static str,
    pub(crate) text: &'static str,
    pub(crate) edits: &'static [(usize, usize, &'static str)],
    pub(crate) expected_text: &'static str,
}
