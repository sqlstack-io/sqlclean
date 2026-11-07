pub(crate) struct CanonicalTokensTestCase {
    pub(crate) description: &'static str,
    pub(crate) dialect: &'static str,
    pub(crate) tokenizer: sqlquack_syntax::tokens::models::Dialect,
    pub(crate) left: &'static str,
    pub(crate) right: &'static str,
    pub(crate) expected_equal: bool,
}
