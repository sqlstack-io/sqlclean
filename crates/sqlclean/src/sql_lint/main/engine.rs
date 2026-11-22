/// Lint one JSON request with the lint engine and return its JSON response.
pub fn lint_json(request_json: &str) -> Result<String, String> {
    sqlclean_lint::lint_json(request_json)
}
