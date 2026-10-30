/// Names a Python interpreter to use instead of the one beside the `sqlclean` executable.
pub const PYTHON_VARIABLE: &str = "SQLCLEAN_PYTHON";
pub(crate) const HOST_PROGRAM: &str =
    "from sqlclean.hosting.main.serve import serve; raise SystemExit(serve())";
pub(crate) const CATALOGUE_MODE: &str = "catalogue";
pub(crate) const EVALUATE_MODE: &str = "evaluate";
pub(crate) const MISSING_PACKAGE_HELP: &str = "Custom Rules need the sqlclean Python package in the environment that runs sqlclean: install it with `pip install sqlclean`, or set SQLCLEAN_PYTHON to an interpreter that has it.";
