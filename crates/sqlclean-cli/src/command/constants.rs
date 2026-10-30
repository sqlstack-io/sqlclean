pub const EXIT_SUCCESS: u8 = 0;
/// Findings remain, or files need changes that `--check` or `--diff` did not write.
pub const EXIT_FINDINGS: u8 = 1;
/// The command could not run: invalid arguments, configuration or input.
pub const EXIT_ERROR: u8 = 2;
/// The directory name of the installed skill under each agent's skills directory.
pub(crate) const SKILL_NAME: &str = "sqlclean";
/// The comment that marks a skill file as written by `sqlclean skills`.
pub(crate) const SKILL_GENERATED_MARKER: &str = "<!-- generated-by: sqlclean skills -->";
/// Each agent target with its project-relative and home-relative skills directory.
pub(crate) const SKILL_TARGETS: [(&str, &str, &str); 3] = [
    ("agents", ".agents/skills", ".agents/skills"),
    ("claude", ".claude/skills", ".claude/skills"),
    ("opencode", ".opencode/skills", ".config/opencode/skills"),
];
/// The targets installed when no `--target` is given; OpenCode also reads `.agents`.
pub(crate) const DEFAULT_SKILL_TARGETS: [&str; 2] = ["agents", "claude"];
/// Only Markdown files are installed and cleaned up.
pub(crate) const SKILL_FILE_EXTENSION: &str = "md";
pub(crate) const SKILL_FILE_SUFFIX: &str = ".md";
