/// Lint one JSON request with SQLQuack's lint engine and return its JSON response.
pub fn lint_json(request_json: &str) -> Result<String, String> {
    sqlquack_lint::lint_json(request_json)
}
