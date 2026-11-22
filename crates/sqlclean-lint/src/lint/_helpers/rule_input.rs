//! What the engine hands every Rule: trees, dialect facts and the request's settings, never SQL or tokens.

use std::collections::HashSet;

use sqlclean_rules::models::{
    DialectFacts, Fixture, MacroOutput, Placeholders, RuleInput, RuleSettings,
};

use crate::constants::{
    GENERATED_PLACEHOLDER_PREFIXES, MACRO_PARAMETER_PLACEHOLDER_PREFIX,
    RELATION_PARAMETER_PLACEHOLDER_PREFIX,
};
use crate::dialects::main::profile::profile;
use crate::dialects::models::LintProfile;
use crate::lint::models::{LintRequest, ParsedSql, RequestFacts};
use crate::quality::constants::DEFAULT_MAX_RANKING_ORDER_BY;

const GENERIC_DIALECTS: [&str; 2] = ["generic", ""];

pub(super) fn request_facts(request: &LintRequest) -> RequestFacts<'_> {
    RequestFacts {
        external: lowered(&request.external_identifiers),
        dependencies: lowered(&request.dependency_identifiers),
        read_by_host: lowered(&request.externally_referenced_ctes),
        macro_outputs: request
            .sql_macro_outputs
            .iter()
            .map(|output| MacroOutput {
                start: output.start,
                end: output.end,
                relations: &output.relations,
            })
            .collect(),
    }
}

pub(super) fn rule_input<'a>(
    request: &'a LintRequest,
    parsed: &'a ParsedSql,
    facts: &'a RequestFacts<'a>,
) -> RuleInput<'a> {
    let profile: LintProfile = profile(parsed.dialect);
    RuleInput {
        statements: &parsed.statements,
        dialect: DialectFacts {
            union_distinct: profile.union_distinct,
            resolved_name: profile.resolved_name,
            same_name: profile.same_name,
            generic: GENERIC_DIALECTS
                .iter()
                .any(|name| request.dialect.eq_ignore_ascii_case(name)),
            plain_call_aggregates: profile.plain_call_aggregates,
            foreign_aggregates: profile.foreign_aggregates,
            named_aggregates: profile.named_aggregates,
            single_argument_aggregates: profile.single_argument_aggregates,
            json_parse_functions: profile.json_parse_functions,
            negated_distinct_from: profile.negated_distinct_from,
        },
        settings: RuleSettings {
            external_identifiers: &facts.external,
            dependency_identifiers: &facts.dependencies,
            externally_referenced_ctes: &facts.read_by_host,
            relation_keys: &request.relation_keys,
            macro_outputs: &facts.macro_outputs,
            fixture: request.fixture_harness.as_ref().map(|harness| Fixture {
                cte_prefixes: &harness.cte_prefixes,
                input_cte_prefixes: &harness.input_cte_prefixes,
                empty_relation: harness.empty_relation.as_deref(),
            }),
            allows_dynamic_output_star: request.allows_dynamic_output_star,
            max_ranking_order_by: request
                .max_ranking_order_by
                .unwrap_or(DEFAULT_MAX_RANKING_ORDER_BY),
            placeholders: Placeholders {
                generated: &GENERATED_PLACEHOLDER_PREFIXES,
                macro_parameter: MACRO_PARAMETER_PLACEHOLDER_PREFIX,
                relation_parameter: RELATION_PARAMETER_PLACEHOLDER_PREFIX,
            },
        },
    }
}

fn lowered(names: &[String]) -> HashSet<String> {
    names.iter().map(|name| name.to_ascii_lowercase()).collect()
}
