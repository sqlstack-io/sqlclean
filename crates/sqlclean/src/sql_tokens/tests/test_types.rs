pub(crate) struct CanonicalTokensTestCase {
    pub(crate) description: &'static str,
    pub(crate) dialect: &'static str,
    pub(crate) tokenizer: sqlquack_syntax::tokens::models::Dialect,
    pub(crate) left: &'static str,
    pub(crate) right: &'static str,
    pub(crate) expected_equal: bool,
}

pub(crate) struct FingerprintGoldenTestCase {
    pub(crate) description: &'static str,
    pub(crate) dialect: &'static str,
    pub(crate) sql: &'static str,
    pub(crate) expected_fingerprint: &'static str,
}

pub(crate) struct FingerprintEqualityTestCase {
    pub(crate) description: &'static str,
    pub(crate) left_dialect: &'static str,
    pub(crate) left: &'static str,
    pub(crate) right_dialect: &'static str,
    pub(crate) right: &'static str,
    pub(crate) expected_equal: bool,
}
