"""Build the SqlFile a Rule receives from the native statement facts."""

from collections.abc import Mapping

from sqlclean.hosting._helpers.sql_files import sql_file_from_facts
from sqlclean.rules.models import SqlFile


def build_sql_file(*, path: str, source: str, dialect: str, facts: Mapping[str, object]) -> SqlFile:
    """Return the file with its statements and their located structures."""

    return sql_file_from_facts(path=path, source=source, dialect=dialect, facts=facts)
