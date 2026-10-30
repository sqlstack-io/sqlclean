/// The characters of `sql` from `start` to `end`.
pub(crate) fn text(sql: &str, start: usize, end: usize) -> String {
    sql.chars().skip(start).take(end - start).collect()
}
