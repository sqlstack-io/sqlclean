pub(crate) struct DiscoveryTestCase {
    pub(crate) description: &'static str,
    pub(crate) explicit_paths: &'static [&'static str],
    pub(crate) expected_files: &'static [(&'static str, &'static str)],
}
