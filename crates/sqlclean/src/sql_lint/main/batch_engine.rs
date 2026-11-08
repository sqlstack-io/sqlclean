/// Lint a JSON array of requests in parallel, with SQLQuack's lint engine.
pub fn lint_batch_json(request_json: &str) -> Result<String, String> {
    sqlquack_lint::lint_batch_json(request_json)
}
