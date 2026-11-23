//! Which Rules run, and the diagnostics of what they find.

use std::borrow::Cow;

use sqlclean_rules::models::{RuleFinding, RuleInput};

use crate::lint::_helpers::engine::{
    AMBIGUOUS_ORDER_DIRECTIONS, COMMENT_ATTACHMENT, CONSECUTIVE_SEMICOLON, CONSTANT_PREDICATE,
    COUNT_ONE, CTE_ONLY_BODY, DUPLICATE_OUTPUT_ALIAS, DUPLICATE_TABLE_ALIAS, EXPLICIT_UNION,
    FINAL_CTE_NAME, IMPLICIT_CARTESIAN_JOIN, IMPLICIT_INNER_JOIN,
    INCONSISTENT_SINGLE_SOURCE_QUALIFICATION, INLINE_QUERY_RELATION, JOIN_EXPRESSION,
    JOIN_WITHOUT_CONDITION, MIXED_GROUP_ORDER_REFERENCES, NESTED_CTE, NESTED_ELSE_CASE,
    NULL_COMPARISON, NULL_NOT_IN, NULL_REJECTED_LEFT_JOIN, PARENTHESIZED_DISTINCT,
    POSITIONAL_SET_STAR, PROJECTED_STAR, RECURSIVE_CTE, REDUNDANT_DISTINCT, REDUNDANT_ELSE_NULL,
    REDUNDANT_SELF_ALIAS, SET_ARITY_MISMATCH, SIMPLE_BOOLEAN_CASE, TERMINAL_SELECT,
    UNALIASED_CALCULATION, UNKNOWN_RELATION_QUALIFIER, UNORDERED_LIMIT,
    UNQUALIFIED_MULTI_SOURCE_COLUMN, UNUSED_CTE, UNUSED_JOINED_RELATION, UNUSED_TABLE_ALIAS,
};
use crate::lint::_helpers::named_reports::named_findings;
use crate::lint::_helpers::rule_edits::rendered;
use crate::lint::_helpers::text_views::{comments, terminators};
use crate::lint::models::{LintDiagnostic, LintEdit, LintRuleMetadata, Report};
use crate::quality::constants as quality;

/// A Rule that reads only the tree.
struct TreeRule {
    rule: &'static LintRuleMetadata,
    find: fn(&RuleInput<'_>) -> Vec<RuleFinding>,
    /// Why a finding has no fix, when it has none.
    refusal: &'static str,
}

const TREE_RULES: [TreeRule; 45] = [
    TreeRule {
        rule: &NULL_COMPARISON,
        find: sqlclean_rules::null_comparisons,
        refusal: "only equality and inequality comparisons have a deterministic repair",
    },
    TreeRule {
        rule: &IMPLICIT_CARTESIAN_JOIN,
        find: sqlclean_rules::implicit_joins,
        refusal: "mixed joins, comments, or generated SQL require an explicit authored repair",
    },
    TreeRule {
        rule: &JOIN_WITHOUT_CONDITION,
        find: sqlclean_rules::joins_without_condition,
        refusal: "qualified joins or joins with placeholder conditions require user intent",
    },
    TreeRule {
        rule: &UNORDERED_LIMIT,
        find: sqlclean_rules::unordered_limits,
        refusal: "deterministic ordering columns and tie-breakers require user intent",
    },
    TreeRule {
        rule: &UNUSED_CTE,
        find: sqlclean_rules::unused_ctes,
        refusal: "only contiguous authored, comment-free, select-only CTEs can be removed",
    },
    TreeRule {
        rule: &REDUNDANT_DISTINCT,
        find: sqlclean_rules::redundant_distincts,
        refusal: "comments beside DISTINCT would be lost",
    },
    TreeRule {
        rule: &POSITIONAL_SET_STAR,
        find: sqlclean_rules::positional_stars,
        refusal: "column names and their intended positional order require resolved schema evidence",
    },
    TreeRule {
        rule: &EXPLICIT_UNION,
        find: sqlclean_rules::bare_unions,
        refusal: "the active dialect does not accept explicit UNION DISTINCT",
    },
    TreeRule {
        rule: &DUPLICATE_TABLE_ALIAS,
        find: sqlclean_rules::duplicate_table_aliases,
        refusal: "choosing a new relation alias requires user intent",
    },
    TreeRule {
        rule: &DUPLICATE_OUTPUT_ALIAS,
        find: sqlclean_rules::duplicate_output_aliases,
        refusal: "choosing a stable output name requires user intent",
    },
    TreeRule {
        rule: &MIXED_GROUP_ORDER_REFERENCES,
        find: sqlclean_rules::mixed_references,
        refusal: "normalization requires reliable projection-name resolution",
    },
    TreeRule {
        rule: &REDUNDANT_ELSE_NULL,
        find: sqlclean_rules::redundant_else_nulls,
        refusal: "comments inside the clause would be lost",
    },
    TreeRule {
        rule: &PARENTHESIZED_DISTINCT,
        find: sqlclean_rules::parenthesized_distincts,
        refusal: "the parentheses group an expression that continues after them, or hold comments",
    },
    TreeRule {
        rule: &CONSTANT_PREDICATE,
        find: sqlclean_rules::constant_predicates,
        refusal: "removing or replacing a scaffold predicate requires user intent",
    },
    TreeRule {
        rule: &REDUNDANT_SELF_ALIAS,
        find: sqlclean_rules::redundant_self_aliases,
        refusal: "removing the alias may change statically inferred output identity",
    },
    TreeRule {
        rule: &COUNT_ONE,
        find: sqlclean_rules::count_ones,
        refusal: "comments inside the count would be lost",
    },
    TreeRule {
        rule: &NULL_NOT_IN,
        find: sqlclean_rules::null_not_ins,
        refusal: "the intended NULL and anti-join semantics require user intent",
    },
    TreeRule {
        rule: &SET_ARITY_MISMATCH,
        find: sqlclean_rules::arity_mismatches,
        refusal: "the intended branch schema requires user intent",
    },
    TreeRule {
        rule: &PROJECTED_STAR,
        find: sqlclean_rules::projected_stars,
        refusal: "column enumeration requires reliable resolved schema evidence",
    },
    TreeRule {
        rule: &UNALIASED_CALCULATION,
        find: sqlclean_rules::unaliased_calculations,
        refusal: "a meaningful stable output name requires user intent",
    },
    TreeRule {
        rule: &UNUSED_TABLE_ALIAS,
        find: sqlclean_rules::unused_table_aliases,
        refusal: "removing the alias would lose a comment or collide with another relation name",
    },
    TreeRule {
        rule: &NULL_REJECTED_LEFT_JOIN,
        find: sqlclean_rules::null_rejected_left_joins,
        refusal: "moving the predicate or changing join type requires user intent",
    },
    TreeRule {
        rule: &IMPLICIT_INNER_JOIN,
        find: sqlclean_rules::implicit_inner_joins,
        refusal: "INNER cannot be written between DIRECTED and JOIN",
    },
    TreeRule {
        rule: &AMBIGUOUS_ORDER_DIRECTIONS,
        find: sqlclean_rules::ambiguous_order_directions,
        refusal: "normalizing every expression requires a multi-range authored rewrite",
    },
    TreeRule {
        rule: &UNQUALIFIED_MULTI_SOURCE_COLUMN,
        find: sqlclean_rules::unqualified_columns,
        refusal: "choosing the correct relation qualifier requires user intent",
    },
    TreeRule {
        rule: &INCONSISTENT_SINGLE_SOURCE_QUALIFICATION,
        find: sqlclean_rules::inconsistent_qualifications,
        refusal: "qualification style is normalized by the canonical formatter only when safe",
    },
    TreeRule {
        rule: &UNKNOWN_RELATION_QUALIFIER,
        find: sqlclean_rules::unknown_qualifiers,
        refusal: "the intended relation or alias requires user intent",
    },
    TreeRule {
        rule: &SIMPLE_BOOLEAN_CASE,
        find: sqlclean_rules::boolean_cases,
        refusal: "comments or a non-canonical CASE shape prevent a lossless rewrite",
    },
    TreeRule {
        rule: &NESTED_ELSE_CASE,
        find: sqlclean_rules::nested_else_cases,
        refusal: "flattening nested branches requires a multi-range authored rewrite",
    },
    TreeRule {
        rule: &UNUSED_JOINED_RELATION,
        find: sqlclean_rules::unused_joined_relations,
        refusal: "the join may intentionally filter or multiply rows",
    },
    TreeRule {
        rule: &CTE_ONLY_BODY,
        find: sqlclean_rules::cte_only_bodies,
        refusal: "extracting transformation logic into named CTEs requires user intent",
    },
    TreeRule {
        rule: &TERMINAL_SELECT,
        find: sqlclean_rules::terminal_selects,
        refusal: "choosing the final logical CTE and projection requires user intent",
    },
    TreeRule {
        rule: &NESTED_CTE,
        find: sqlclean_rules::nested_ctes,
        refusal: "hoisting a nested CTE requires an authored query rewrite",
    },
    TreeRule {
        rule: &RECURSIVE_CTE,
        find: sqlclean_rules::recursive_ctes,
        refusal: "replacing recursion requires an authored query design",
    },
    TreeRule {
        rule: &INLINE_QUERY_RELATION,
        find: sqlclean_rules::inline_query_relations,
        refusal: "extracting a derived relation can change correlation and scope",
    },
    TreeRule {
        rule: &JOIN_EXPRESSION,
        find: sqlclean_rules::computed_conditions,
        refusal: "computing a join key requires an authored CTE rewrite",
    },
    TreeRule {
        rule: &FINAL_CTE_NAME,
        find: sqlclean_rules::misnamed_final_ctes,
        refusal: "renaming a CTE requires updating its references",
    },
    TreeRule {
        rule: &quality::DUPLICATE_CTE_NAME,
        find: sqlclean_rules::duplicate_cte_names,
        refusal: "Renaming a CTE requires choosing which definition its readers mean",
    },
    TreeRule {
        rule: &quality::NULL_MATCH,
        find: sqlclean_rules::null_matches,
        refusal: "Rewriting to IS NULL changes what the query returns",
    },
    TreeRule {
        rule: &quality::LITERAL_ZERO_DIVISION,
        find: sqlclean_rules::literal_zero_divisions,
        refusal: "Choosing a divisor or a NULL guard requires the intended result",
    },
    TreeRule {
        rule: &quality::UNFILTERED_WRITE,
        find: sqlclean_rules::unfiltered_writes,
        refusal: "Only the author knows which rows the statement should change",
    },
    TreeRule {
        rule: &quality::INSERT_WITHOUT_COLUMNS,
        find: sqlclean_rules::inserts_without_columns,
        refusal: "Listing the target columns requires the table's schema",
    },
    TreeRule {
        rule: &quality::NATURAL_JOIN,
        find: sqlclean_rules::natural_joins,
        refusal: "Writing the join condition requires both relations' schemas",
    },
    TreeRule {
        rule: &quality::REVERSED_BETWEEN,
        find: sqlclean_rules::reversed_betweens,
        refusal: "Swapping the bounds changes what the query returns",
    },
    TreeRule {
        rule: &quality::MISPLACED_AGGREGATE,
        find: sqlclean_rules::misplaced_aggregates,
        refusal: "Moving the expression to HAVING, QUALIFY or an outer query requires restructuring the query",
    },
];
const MISPLACED_COMMENT_REFUSAL: &str =
    "comment attachment requires an authored placement decision";

/// Every enabled Rule's diagnostics.
pub(super) fn reported(report: &Report<'_>) -> Vec<LintDiagnostic> {
    let mut found: Vec<LintDiagnostic> = named_findings(report);
    for tree_rule in &TREE_RULES {
        if report.enabled.contains(tree_rule.rule.code) {
            let findings: Vec<RuleFinding> = (tree_rule.find)(report.input);
            found.extend(diagnostics(
                report,
                (tree_rule.rule, Some(tree_rule.refusal)),
                &findings,
            ));
        }
    }
    let tokens = &report.parsed.tokens;
    if report.enabled.contains(CONSECUTIVE_SEMICOLON.code) {
        let findings = sqlclean_rules::consecutive_semicolons(&terminators(tokens));
        found.extend(diagnostics(
            report,
            (&CONSECUTIVE_SEMICOLON, None),
            &findings,
        ));
    }
    if report.enabled.contains(COMMENT_ATTACHMENT.code) {
        let placed = comments(tokens, &report.request.sql);
        let findings = sqlclean_rules::misplaced_comments(report.input, &placed);
        found.extend(diagnostics(
            report,
            (&COMMENT_ATTACHMENT, Some(MISPLACED_COMMENT_REFUSAL)),
            &findings,
        ));
    }
    found
}

fn diagnostics(
    report: &Report<'_>,
    (rule, refusal): (&LintRuleMetadata, Option<&'static str>),
    findings: &[RuleFinding],
) -> Vec<LintDiagnostic> {
    findings
        .iter()
        .map(|finding| {
            let fix: Option<LintEdit> = match &finding.fix {
                Some(edit) => rendered(edit, &report.request.sql, &report.parsed.tokens),
                None => None,
            };
            LintDiagnostic {
                code: rule.code,
                message: Cow::Borrowed(rule.message),
                remediation: Cow::Borrowed(rule.remediation),
                start: finding.span.start,
                end: finding.span.end,
                fix_unavailable_reason: if fix.is_none() { refusal } else { None },
                fix,
            }
        })
        .collect()
}
