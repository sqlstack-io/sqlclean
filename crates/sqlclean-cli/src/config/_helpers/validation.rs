//! Checks that need more than the schema: positive limits, valid globs, dialects and reasons.

use globset::Glob;
use sqlclean::sql_lint::main::validate_dialect::validate_dialect;

use crate::config::models::Config;

pub(crate) fn validate(config: &Config) -> Result<(), String> {
    validate_dialect(&config.dialect)?;
    for dialect in &config.dialects {
        validate_dialect(&dialect.dialect)?;
        validate_globs(&dialect.paths)?;
    }
    validate_globs(&config.include)?;
    validate_globs(&config.exclude)?;
    if config.line_width == 0 {
        return Err("[format] line_width must be positive".to_owned());
    }
    let thresholds = config.rules.thresholds;
    if thresholds.max_literal_length == 0 || thresholds.max_ranking_order_by == 0 {
        return Err(
            "[rules.thresholds] max_literal_length and max_ranking_order_by must be positive"
                .to_owned(),
        );
    }
    for exception in &config.rules.rule_exceptions {
        require_reason(&exception.reason, &exception.rule)?;
        if exception.relation.is_none() && exception.cte.is_some() {
            return Err(format!(
                "rule exception for {} at {} sets cte without relation",
                exception.rule, exception.path
            ));
        }
    }
    for ignore in &config.rules.rule_ignores {
        require_reason(&ignore.reason, &ignore.rules.join(", "))?;
        validate_globs(&ignore.paths)?;
    }
    Ok(())
}

fn validate_globs(patterns: &[String]) -> Result<(), String> {
    for pattern in patterns {
        Glob::new(pattern).map_err(|error| format!("invalid path pattern '{pattern}': {error}"))?;
    }
    Ok(())
}

fn require_reason(reason: &str, subject: &str) -> Result<(), String> {
    if reason.trim().is_empty() {
        return Err(format!(
            "rule exception or ignore for {subject} needs a reason"
        ));
    }
    Ok(())
}
