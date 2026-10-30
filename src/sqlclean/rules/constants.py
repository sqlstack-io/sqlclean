"""Constants of the custom Rule authoring API."""

RULE_ATTRIBUTE: str = "__sqlclean_rule__"
CUSTOM_CODE_PREFIX: str = "XSQC"
RULE_CODE_DIGIT_COUNT: int = 3
RULE_PARAMETER_COUNT: int = 2
SIGNATURE_REQUIREMENT: str = (
    "must accept exactly one typed Statement, SqlFile or Repository subject and one typed "
    "RuleContext, both keyword-only"
)
POLYGLOT_MODULE: str = "polyglot_sql"
POLYGLOT_INSTALL_HINT: str = "install the ast extra: pip install 'sqlclean[ast]'"
