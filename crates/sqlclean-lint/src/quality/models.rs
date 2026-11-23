//! What the quality Rules' words are made of.

/// One setting a remediation tells the user to write.
#[derive(Debug)]
pub(crate) struct Setting<'a> {
    pub(crate) purpose: &'a str,
    pub(crate) config_file: &'a str,
    pub(crate) section: &'a str,
    pub(crate) key: &'a str,
    pub(crate) value: &'a str,
}
