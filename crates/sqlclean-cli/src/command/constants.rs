pub const EXIT_SUCCESS: u8 = 0;
/// Findings remain, or files need changes that `--check` or `--diff` did not write.
pub const EXIT_FINDINGS: u8 = 1;
/// The command could not run: invalid arguments, configuration or input.
pub const EXIT_ERROR: u8 = 2;
