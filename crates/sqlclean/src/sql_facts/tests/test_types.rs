pub(crate) struct StatementFactsTestCase {
    pub(crate) description: &'static str,
    pub(crate) sql: &'static str,
    pub(crate) statement: usize,
    pub(crate) expected_kind: &'static str,
    pub(crate) expected_text: &'static str,
    pub(crate) expected_terminator: &'static str,
    pub(crate) expected_ctes: &'static [&'static str],
    pub(crate) expected_tables: &'static [(&'static str, &'static str)],
    pub(crate) expected_joins: &'static [(&'static str, &'static str, bool, &'static str)],
    pub(crate) expected_stars: &'static [&'static str],
    pub(crate) expected_functions: &'static [&'static str],
    pub(crate) expected_literals: &'static [&'static str],
    pub(crate) expected_columns: &'static [(&'static str, &'static str)],
}

pub(crate) struct ParseOutcomeTestCase {
    pub(crate) description: &'static str,
    pub(crate) sql: &'static str,
    pub(crate) expected_statements: Option<usize>,
}
