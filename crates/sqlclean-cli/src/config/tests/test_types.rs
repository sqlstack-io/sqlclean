pub(crate) struct LoadedConfigTestCase {
    pub(crate) description: &'static str,
    pub(crate) config_text: &'static str,
    pub(crate) start: &'static str,
    pub(crate) expected_dialect: &'static str,
    pub(crate) expected_line_width: usize,
    pub(crate) expected_select: Option<&'static [&'static str]>,
    pub(crate) expected_max_literal_length: usize,
    pub(crate) expected_max_ranking_order_by: usize,
}

pub(crate) struct RejectedConfigTestCase {
    pub(crate) description: &'static str,
    pub(crate) config_text: &'static str,
    pub(crate) expected_error: &'static str,
}
