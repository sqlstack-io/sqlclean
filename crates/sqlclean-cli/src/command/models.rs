//! The command-line interface.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::reporting::models::OutputFormat;

/// Fast, opinionated SQL linting and deterministic formatting.
#[derive(Debug, Parser)]
#[command(name = "sqlclean", version, about)]
pub struct Cli {
    /// Use this configuration file instead of the nearest sqlclean.toml.
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Report Rule findings without changing any file.
    Check(CheckArgs),
    /// Apply Rule fixes until stable, then format.
    Fix(FixArgs),
    /// Rewrite SQL files in the canonical layout.
    Format(FormatArgs),
    /// List the available Rules, or show one.
    Rules(RulesArgs),
    /// Print the structural facts custom Rules read for SQL on standard input, as JSON.
    Facts(FactsArgs),
}

/// Rule selection shared by check and fix.
#[derive(Debug, Args)]
pub struct SelectionArgs {
    /// Files or directories to read; the configuration root when omitted.
    pub paths: Vec<PathBuf>,
    /// Rule codes or families to run instead of the configured selection.
    #[arg(long, value_delimiter = ',')]
    pub select: Option<Vec<String>>,
    /// Rule codes or families to skip, in addition to the configured ones.
    #[arg(long, value_delimiter = ',')]
    pub ignore: Vec<String>,
    /// Evaluate every file instead of reusing results for unchanged inputs.
    #[arg(long)]
    pub no_cache: bool,
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    #[command(flatten)]
    pub selection: SelectionArgs,
    #[arg(long, value_enum, default_value = "text")]
    pub output_format: OutputFormat,
}

#[derive(Debug, Args)]
pub struct FixArgs {
    #[command(flatten)]
    pub selection: SelectionArgs,
    /// Also apply fixes their Rules mark unsafe because they can change results.
    #[arg(long)]
    pub unsafe_fixes: bool,
    /// Print the changes as a unified diff instead of writing them.
    #[arg(long)]
    pub diff: bool,
}

#[derive(Debug, Args)]
pub struct FormatArgs {
    /// Files or directories to format; the configuration root when omitted.
    pub paths: Vec<PathBuf>,
    /// Fail when a file needs formatting, without writing it.
    #[arg(long)]
    pub check: bool,
    /// Print the changes as a unified diff instead of writing them.
    #[arg(long)]
    pub diff: bool,
}

#[derive(Debug, Args)]
pub struct RulesArgs {
    /// Show one Rule's details.
    pub code: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct FactsArgs {
    #[arg(long, default_value = "generic")]
    pub dialect: String,
}
