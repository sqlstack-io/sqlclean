pub(crate) struct CanonicalTokensTestCase {
    pub(crate) description: &'static str,
    pub(crate) dialect: &'static str,
    pub(crate) left: &'static str,
    pub(crate) right: &'static str,
    pub(crate) expected_equal: bool,
}
