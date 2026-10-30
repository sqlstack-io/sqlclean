//! Constants shared by the engine modules, and the placeholders hosts write for template code.

pub(crate) const SQL_WILDCARD: &str = "*";
pub(crate) const OPEN_PAREN: &str = "(";
pub(crate) const CLOSE_PAREN: &str = ")";
pub(crate) const EMPTY_FIXTURE_QUERY_TOKEN_COUNT: usize = 6;
pub(crate) const ENCLOSING_CTE_TOKEN_COUNT: usize = 3;
/// The indent of each line of a configuration snippet printed in remediation text.
pub(crate) const SETTING_SNIPPET_INDENT: &str = "            ";
/// The configuration file remediation text names when the request does not name one.
pub const DEFAULT_CONFIG_FILE: &str = "sqlclean.toml";
/// The identifier prefix a host writes in place of each template interpolation.
pub const LINT_SENTINEL_PREFIX: &str = "__sqb_lint_";
/// The identifier prefix a host writes in place of a SQL macro parameter.
pub const SQL_MACRO_PARAMETER_SENTINEL_PREFIX: &str = "__sqlbuild_macro_parameter_";
