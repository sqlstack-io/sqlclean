pub(crate) struct CacheRunTestCase {
    pub(crate) description: &'static str,
    pub(crate) existing_cache: &'static str,
    pub(crate) enabled: bool,
    pub(crate) puts: &'static [(&'static str, &'static str, i32)],
    pub(crate) read_paths: &'static [&'static str],
    pub(crate) gets: &'static [&'static str],
    pub(crate) expected_file_rewritten: bool,
    pub(crate) expected_reopened: &'static [(&'static str, Option<i32>)],
}

pub(crate) struct CacheKeyTestCase {
    pub(crate) description: &'static str,
    pub(crate) left: &'static [&'static str],
    pub(crate) right: &'static [&'static str],
    pub(crate) expected_equal: bool,
}
