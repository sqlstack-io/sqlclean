//! `sqlclean rules`.

use std::fmt::Write as _;
use std::io::Write;

use crate::checking::main::rule_catalogue::rule_catalogue;
use crate::checking::models::RuleInfo;
use crate::command::constants::{EXIT_ERROR, EXIT_SUCCESS};
use crate::command::models::RulesArgs;
use crate::config::models::Config;

pub(crate) fn run(config: &Config, arguments: &RulesArgs) -> Result<u8, String> {
    let rules: Vec<RuleInfo> = rule_catalogue(config, true)?;
    let shown: Vec<&RuleInfo> = match &arguments.code {
        Some(code) => rules.iter().filter(|rule| &rule.code == code).collect(),
        None => rules.iter().collect(),
    };
    if shown.is_empty() {
        let _ = writeln!(std::io::stderr(), "sqlclean: error: unknown Rule code");
        return Ok(EXIT_ERROR);
    }
    let output: String = if arguments.json {
        serde_json::to_string_pretty(&shown).map_err(|error| error.to_string())? + "\n"
    } else if arguments.code.is_some() {
        detail(shown[0])
    } else {
        table(&shown)
    };
    let _ = std::io::stdout().write_all(output.as_bytes());
    Ok(EXIT_SUCCESS)
}

fn table(rules: &[&RuleInfo]) -> String {
    let mut output = String::new();
    for rule in rules {
        let default: &str = if rule.enabled_by_default {
            "default"
        } else {
            ""
        };
        let _ = writeln!(output, "{:<12} {:<8} {}", rule.code, default, rule.message);
    }
    output
}

fn detail(rule: &RuleInfo) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "{}: {}", rule.code, rule.message);
    let _ = writeln!(
        output,
        "Subject: {}",
        serde_json::to_value(rule.subject)
            .unwrap_or_default()
            .as_str()
            .unwrap_or_default()
    );
    let _ = writeln!(
        output,
        "Enabled by default: {}",
        if rule.enabled_by_default { "yes" } else { "no" }
    );
    if let Some(module) = &rule.module {
        let _ = writeln!(output, "Defined in: {module}");
    }
    if let Some(fixable) = rule.fixable {
        let _ = writeln!(
            output,
            "Fixable: {}",
            serde_json::to_value(fixable)
                .unwrap_or_default()
                .as_str()
                .unwrap_or_default()
        );
    }
    let _ = writeln!(output, "Remediation: {}", rule.remediation);
    output
}
