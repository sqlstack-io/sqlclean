//! The placeholders hosts write for template code, the same values SQLQuack's lint engine reads.

use sqlquack_lint::constants as lint;

/// The configuration file remediation text names when the request does not name one.
pub const DEFAULT_CONFIG_FILE: &str = lint::DEFAULT_CONFIG_FILE;
/// The identifier prefix a host writes in place of each template interpolation.
pub const INTERPOLATION_PLACEHOLDER_PREFIX: &str = lint::INTERPOLATION_PLACEHOLDER_PREFIX;
/// The identifier prefix a host writes in place of a SQL macro parameter.
pub const MACRO_PARAMETER_PLACEHOLDER_PREFIX: &str = lint::MACRO_PARAMETER_PLACEHOLDER_PREFIX;
/// The prefix of a relation parameter placeholder, written `<prefix><index>__`.
pub const RELATION_PARAMETER_PLACEHOLDER_PREFIX: &str = lint::RELATION_PARAMETER_PLACEHOLDER_PREFIX;
/// The identifier prefix a host writes in place of a value parameter.
pub const VALUE_PARAMETER_PLACEHOLDER_PREFIX: &str = lint::VALUE_PARAMETER_PLACEHOLDER_PREFIX;
