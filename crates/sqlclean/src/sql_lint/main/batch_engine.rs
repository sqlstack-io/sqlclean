/// Lint a JSON array of requests in parallel, with the lint engine.
pub fn lint_batch_json(request_json: &str) -> Result<String, String> {
    sqlclean_lint::lint_batch_json(request_json)
}
