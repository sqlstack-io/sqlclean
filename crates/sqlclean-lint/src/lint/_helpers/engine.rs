use std::borrow::Cow;
use std::collections::HashSet;

use sqlclean_rules::models::RuleInput;
use sqlquack_syntax::{CallMode, Expression};

use crate::dialects::main::lint_dialect::lint_dialect;
use crate::lint::_helpers::parsing::parse;
use crate::lint::_helpers::report::reported;
use crate::lint::_helpers::rule_input::{request_facts, rule_input};
use crate::lint::_helpers::trigger_programs::with_trigger_programs;
use crate::lint::constants::LINT_API_VERSION;
use crate::lint::models::{
    LintRequest, LintResponse, LintRuleMetadata, ParsedSql, Report, RequestFacts,
};

pub(super) const NULL_COMPARISON: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN001",
    message: "Comparison with NULL is never true",
    remediation: "Use IS NULL or IS NOT NULL when testing for NULL.",
};

pub(super) const IMPLICIT_CARTESIAN_JOIN: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN002",
    message: "Comma-separated sources create an implicit cartesian join",
    remediation: "Replace comma-separated sources with an explicit keyed join, or use CROSS JOIN when the cartesian product is intentional.",
};
pub(super) const JOIN_WITHOUT_CONDITION: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN003",
    message: "Non-cross join has no meaningful condition",
    remediation: "Add a meaningful ON or USING condition, or declare an intentional cartesian product with CROSS JOIN.",
};
pub(super) const UNORDERED_LIMIT: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN004",
    message: "Row selection is nondeterministic",
    remediation: "Add ORDER BY with a deterministic tie-breaker before LIMIT or OFFSET.",
};
pub(super) const UNUSED_CTE: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN005",
    message: "CTE is unreachable from the final query",
    remediation: "Reference the CTE from the final query or another reachable CTE, or remove it.",
};
pub(super) const REDUNDANT_DISTINCT: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN006",
    message: "DISTINCT is redundant with the grouped output",
    remediation: "Remove DISTINCT; the equivalent GROUP BY already determines the output groups.",
};
pub(super) const POSITIONAL_SET_STAR: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN007",
    message: "Positional set operation is vulnerable to column-order drift",
    remediation: "Enumerate columns in the same order in every set-operation branch.",
};
pub(super) const EXPLICIT_UNION: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN008",
    message: "Bare UNION leaves duplicate handling implicit",
    remediation: "Use UNION DISTINCT or UNION ALL to state duplicate handling explicitly.",
};
pub(super) const DUPLICATE_TABLE_ALIAS: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN009",
    message: "Table alias is duplicated in the same query scope",
    remediation: "Give every relation in this query scope a unique alias.",
};
pub(super) const DUPLICATE_OUTPUT_ALIAS: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN010",
    message: "Output alias is duplicated in the same select list",
    remediation: "Give every projected expression a unique output alias.",
};
pub(super) const MIXED_GROUP_ORDER_REFERENCES: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN011",
    message: "Clause mixes positional and named references",
    remediation: "Use column names or positions consistently throughout this clause.",
};
pub(super) const REDUNDANT_ELSE_NULL: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN012",
    message: "ELSE NULL is redundant",
    remediation: "Remove ELSE NULL; CASE expressions return NULL when ELSE is omitted.",
};
pub(super) const PARENTHESIZED_DISTINCT: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN013",
    message: "DISTINCT is written as though it were a function",
    remediation: "Write DISTINCT as a SELECT modifier without function-like parentheses.",
};
pub(super) const CONSTANT_PREDICATE: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN014",
    message: "Predicate compares identical constant values",
    remediation: "Remove the scaffold predicate or replace it with the intended condition.",
};
pub(super) const CONSECUTIVE_SEMICOLON: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN015",
    message: "Empty statement follows another terminator",
    remediation: "Remove the redundant statement terminator.",
};
pub(super) const REDUNDANT_SELF_ALIAS: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN016",
    message: "Expression is aliased to its existing name",
    remediation: "Remove the self-alias when the inferred output identity remains stable.",
};
pub(super) const COUNT_ONE: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN017",
    message: "Row count uses a constant expression",
    remediation: "Use COUNT(*) to state that rows, rather than a nullable expression, are counted.",
};
pub(super) const UNSTABLE_ROW_NUMBER: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN018",
    message: "ROW_NUMBER has no deterministic ordering",
    remediation: "Add ORDER BY with a stable tie-breaker inside the window specification.",
};
pub(super) const NULL_NOT_IN: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN019",
    message: "NOT IN list contains NULL and can reject every row",
    remediation: "Remove NULL from the list or express the anti-join with a NULL-safe NOT EXISTS.",
};
pub(super) const SET_ARITY_MISMATCH: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN020",
    message: "Set-operation branches project different column counts",
    remediation: "Project the same number of columns in every set-operation branch.",
};
pub(super) const PROJECTED_STAR: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN021",
    message: "Query output has an uncontrolled column shape",
    remediation: "Enumerate the intended output columns explicitly.",
};
pub(super) const UNALIASED_CALCULATION: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN022",
    message: "Calculated projection has no stable output name",
    remediation: "Add an explicit AS alias that describes the projected value.",
};
pub(super) const UNUSED_TABLE_ALIAS: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN023",
    message: "Table alias is never referenced",
    remediation: "Use the alias to qualify references or remove it.",
};
pub(super) const NULL_REJECTED_LEFT_JOIN: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN024",
    message: "WHERE predicate rejects NULL rows from a LEFT JOIN",
    remediation: "Move the right-side predicate into ON or use INNER JOIN when row rejection is intentional.",
};
pub(super) const IMPLICIT_INNER_JOIN: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN025",
    message: "JOIN type is implicit",
    remediation: "Write INNER JOIN explicitly.",
};
pub(super) const AMBIGUOUS_ORDER_DIRECTIONS: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN026",
    message: "ORDER BY mixes explicit and implicit directions",
    remediation: "State ASC or DESC for every ordering expression in this clause.",
};
pub(super) const UNQUALIFIED_MULTI_SOURCE_COLUMN: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN027",
    message: "Column reference is unqualified in a multi-relation query",
    remediation: "Qualify the column with its relation alias.",
};
pub(super) const INCONSISTENT_SINGLE_SOURCE_QUALIFICATION: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN028",
    message: "Single-relation query mixes qualified and unqualified columns",
    remediation: "Use one consistent qualification style in this query scope.",
};
pub(super) const UNKNOWN_RELATION_QUALIFIER: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN029",
    message: "Qualified reference uses a relation absent from this query scope",
    remediation: "Use an available relation alias or add the intended relation to FROM.",
};
pub(super) const SIMPLE_BOOLEAN_CASE: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN030",
    message: "CASE expression only converts a condition to TRUE or FALSE",
    remediation: "Use a NULL-safe boolean expression directly.",
};
pub(super) const NESTED_ELSE_CASE: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN031",
    message: "CASE expression is nested directly inside ELSE",
    remediation: "Flatten the nested branch into the surrounding CASE expression.",
};
pub(super) const UNUSED_JOINED_RELATION: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN032",
    message: "Joined relation contributes no referenced values",
    remediation: "Use the joined relation or remove the join. When the join intentionally filters rows, record it as a reviewed rule exception with a reason in the configuration file.",
};
pub(super) const COMMENT_ATTACHMENT: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN033",
    message: "Comment is not attached to the syntax node it explains",
    remediation: "Use a standalone -- comment block immediately above the relevant CTE, JOIN, filter, predicate, selected expression, CASE branch, grouping, or ordering node.",
};
pub(super) const CTE_ONLY_BODY: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN034",
    message: "Query keeps transformation logic outside top-level CTEs",
    remediation: "Move transformation logic into named top-level CTEs before the terminal SELECT.",
};
pub(super) const TERMINAL_SELECT: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN035",
    message: "Terminal SELECT does not read plainly from the final top-level CTE",
    remediation: "Name the final logical CTE and leave one terminal SELECT from that CTE.",
};
pub(super) const NESTED_CTE: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN036",
    message: "CTE is nested inside another query",
    remediation: "Hoist the nested CTE into the statement's top-level WITH list.",
};
pub(super) const RECURSIVE_CTE: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN037",
    message: "Recursive CTE is not permitted",
    remediation: "Replace recursion with an explicit upstream relation or a bounded non-recursive query shape.",
};
pub(super) const CROSS_JOIN: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN038",
    message: "CROSS JOIN creates an explicit cartesian product",
    remediation: "Use a keyed JOIN, or make one side provably at most one row (LIMIT 1, an aggregate without GROUP BY, or a one-row VALUES). When the cartesian product is intentional, record it as a reviewed [[rules.rule_exceptions]] entry with rule, path, relation and reason in the configuration file.",
};
pub(super) const INLINE_QUERY_RELATION: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN039",
    message: "Inline query relation obscures data flow",
    remediation: "Extract the derived relation into a named top-level CTE; use explicit LATERAL only when the relation must remain correlated.",
};
pub(super) const JOIN_EXPRESSION: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN040",
    message: "JOIN ON contains a computed expression",
    remediation: "Compute keys, including variant path access, in input CTEs and join on plain columns. Literal-only predicates such as ON TRUE are allowed.",
};
pub(super) const FINAL_CTE_NAME: LintRuleMetadata = LintRuleMetadata {
    code: "SQLCLEAN041",
    message: "Only the last top-level CTE may be named final, and it must use that name",
    remediation: "Name the last top-level CTE final and rename any other final CTE; SQLCLEAN035 checks the terminal SELECT.",
};

pub(crate) const DEFAULT_RULES: [&str; 19] = [
    NULL_COMPARISON.code,
    IMPLICIT_CARTESIAN_JOIN.code,
    JOIN_WITHOUT_CONDITION.code,
    UNORDERED_LIMIT.code,
    UNUSED_CTE.code,
    REDUNDANT_DISTINCT.code,
    POSITIONAL_SET_STAR.code,
    DUPLICATE_TABLE_ALIAS.code,
    DUPLICATE_OUTPUT_ALIAS.code,
    UNSTABLE_ROW_NUMBER.code,
    NULL_NOT_IN.code,
    SET_ARITY_MISMATCH.code,
    INLINE_QUERY_RELATION.code,
    crate::quality::constants::DUPLICATE_CTE_NAME.code,
    crate::quality::constants::NULL_MATCH.code,
    crate::quality::constants::LITERAL_ZERO_DIVISION.code,
    crate::quality::constants::NATURAL_JOIN.code,
    crate::quality::constants::REVERSED_BETWEEN.code,
    crate::quality::constants::MISPLACED_AGGREGATE.code,
];

const ALL_RULE_METADATA: [&LintRuleMetadata; 53] = [
    &NULL_COMPARISON,
    &IMPLICIT_CARTESIAN_JOIN,
    &JOIN_WITHOUT_CONDITION,
    &UNORDERED_LIMIT,
    &UNUSED_CTE,
    &REDUNDANT_DISTINCT,
    &POSITIONAL_SET_STAR,
    &EXPLICIT_UNION,
    &DUPLICATE_TABLE_ALIAS,
    &DUPLICATE_OUTPUT_ALIAS,
    &MIXED_GROUP_ORDER_REFERENCES,
    &REDUNDANT_ELSE_NULL,
    &PARENTHESIZED_DISTINCT,
    &CONSTANT_PREDICATE,
    &CONSECUTIVE_SEMICOLON,
    &REDUNDANT_SELF_ALIAS,
    &COUNT_ONE,
    &UNSTABLE_ROW_NUMBER,
    &NULL_NOT_IN,
    &SET_ARITY_MISMATCH,
    &PROJECTED_STAR,
    &UNALIASED_CALCULATION,
    &UNUSED_TABLE_ALIAS,
    &NULL_REJECTED_LEFT_JOIN,
    &IMPLICIT_INNER_JOIN,
    &AMBIGUOUS_ORDER_DIRECTIONS,
    &UNQUALIFIED_MULTI_SOURCE_COLUMN,
    &INCONSISTENT_SINGLE_SOURCE_QUALIFICATION,
    &UNKNOWN_RELATION_QUALIFIER,
    &SIMPLE_BOOLEAN_CASE,
    &NESTED_ELSE_CASE,
    &UNUSED_JOINED_RELATION,
    &COMMENT_ATTACHMENT,
    &CTE_ONLY_BODY,
    &TERMINAL_SELECT,
    &NESTED_CTE,
    &RECURSIVE_CTE,
    &CROSS_JOIN,
    &INLINE_QUERY_RELATION,
    &JOIN_EXPRESSION,
    &FINAL_CTE_NAME,
    &crate::quality::constants::UNUSED_CTE_OUTPUT,
    &crate::quality::constants::RANKING_SORT_CAP,
    &crate::quality::constants::LONG_LITERAL,
    &crate::quality::constants::REPEATED_JSON_PARSE,
    &crate::quality::constants::DUPLICATE_CTE_NAME,
    &crate::quality::constants::NULL_MATCH,
    &crate::quality::constants::LITERAL_ZERO_DIVISION,
    &crate::quality::constants::UNFILTERED_WRITE,
    &crate::quality::constants::INSERT_WITHOUT_COLUMNS,
    &crate::quality::constants::NATURAL_JOIN,
    &crate::quality::constants::REVERSED_BETWEEN,
    &crate::quality::constants::MISPLACED_AGGREGATE,
];

pub(crate) fn lint_json_impl(request_json: &str) -> Result<String, String> {
    let request: LintRequest =
        serde_json::from_str(request_json).map_err(|error| error.to_string())?;
    let response = lint(request)?;
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

pub(crate) fn rule_metadata() -> Vec<LintRuleMetadata> {
    ALL_RULE_METADATA.iter().map(|rule| **rule).collect()
}

pub(crate) fn lint(request: LintRequest) -> Result<LintResponse, String> {
    if request.version != LINT_API_VERSION {
        return Err(format!(
            "unsupported native lint request version {}; expected {LINT_API_VERSION}",
            request.version
        ));
    }
    let enabled = enabled_rules(request.enabled_rules.clone(), &request.ignored_rules)?;
    let Some(dialect) = lint_dialect(&request.dialect)? else {
        return Ok(LintResponse {
            version: LINT_API_VERSION,
            diagnostics: Vec::new(),
            unavailable: Some(format!(
                "SQL lint is unavailable for the {} dialect: SQLQuack does not parse it yet",
                request.dialect
            )),
        });
    };
    let empty_relation: Option<&str> = request
        .fixture_harness
        .as_ref()
        .and_then(|harness| harness.empty_relation.as_deref());
    let parsed: ParsedSql = parse(
        &request.sql,
        dialect,
        (
            empty_relation,
            request.max_function_call_depth,
            CallMode::AsWrittenLocated,
        ),
    )?;
    if request.max_literal_length == Some(0) || request.max_ranking_order_by == Some(0) {
        return Err("max_literal_length and max_ranking_order_by must be positive".to_owned());
    }
    let facts: RequestFacts<'_> = request_facts(&request);
    let statements: Cow<'_, [Expression]> = with_trigger_programs(&parsed.statements);
    let input: RuleInput<'_> = rule_input(&request, &parsed, &facts, &statements);
    let mut diagnostics = reported(&Report {
        request: &request,
        parsed: &parsed,
        enabled: &enabled,
        input: &input,
    });
    diagnostics.sort_by_key(|diagnostic| (diagnostic.start, diagnostic.end, diagnostic.code));
    Ok(LintResponse {
        version: LINT_API_VERSION,
        diagnostics,
        unavailable: None,
    })
}

fn enabled_rules(
    requested: Option<Vec<String>>,
    ignored: &[String],
) -> Result<HashSet<String>, String> {
    let selectors =
        requested.unwrap_or_else(|| DEFAULT_RULES.iter().map(ToString::to_string).collect());
    let mut rules = expand_rule_selectors(&selectors)?;
    for code in expand_rule_selectors(ignored)? {
        rules.remove(&code);
    }
    Ok(rules)
}

fn expand_rule_selectors(selectors: &[String]) -> Result<HashSet<String>, String> {
    let mut selected: HashSet<String> = HashSet::new();
    for selector in selectors {
        let matches: Vec<&str> = ALL_RULE_METADATA
            .iter()
            .map(|rule| rule.code)
            .filter(|code| code.starts_with(selector))
            .collect();
        if matches.is_empty() {
            return Err(format!("unknown native lint rule '{selector}'"));
        }
        selected.extend(matches.into_iter().map(ToString::to_string));
    }
    Ok(selected)
}
