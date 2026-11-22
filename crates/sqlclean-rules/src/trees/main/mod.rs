//! Readings of a tree.

pub(crate) mod call_name;
pub(crate) mod call_positions;
pub(crate) mod contains_aggregate;
pub(crate) mod is_aggregate;
pub(crate) mod is_foreign_aggregate;
pub(crate) mod is_generated;
pub(crate) mod is_row_number;
pub(crate) mod join_chains;
pub(crate) mod node_joins;
pub(crate) mod number_text;
pub(crate) mod placed_queries;
pub(crate) mod principal_query;
pub(crate) mod selects;
pub(crate) mod statement_nodes;
pub(crate) mod unwrapped;
pub(crate) mod walk_in_scope;
