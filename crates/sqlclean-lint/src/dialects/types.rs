//! Types of the dialect profiles.

use sqlquack_syntax::tokens::models::Token;
use sqlquack_syntax::{CallMode, Expression, ParseOptions};

/// The tokens of some SQL as a dialect reads them, or why it cannot.
pub(crate) type ReadTokens = fn(&str) -> Result<Vec<Token>, String>;
/// The statements of some SQL as a dialect parses them, under the shared parser's guards, or why it cannot.
pub(crate) type ReadStatements =
    fn(&str, &ParseOptions, CallMode) -> Result<Vec<Expression>, String>;
#[cfg(feature = "native-parsers")]
/// A dialect's own strict parse of some SQL; on refusal, its byte position and parse message, if any.
pub(crate) type StrictParse = fn(&str) -> Result<Vec<Expression>, (usize, Option<String>)>;
