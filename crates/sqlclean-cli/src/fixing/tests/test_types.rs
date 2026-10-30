pub(crate) struct EditSelectionTestCase {
    pub(crate) description: &'static str,
    pub(crate) unsafe_fixes: bool,
    pub(crate) expected_spans: &'static [(usize, usize)],
}
