//! The Python interpreter that runs custom Rules.

use std::env;
use std::path::PathBuf;

use crate::hosting::constants::PYTHON_VARIABLE;

const SIBLING_NAMES: [&str; 3] = ["python", "python3", "python.exe"];
const PATH_FALLBACK: &str = "python3";

/// `SQLCLEAN_PYTHON`, else the interpreter installed beside this executable, else `python3`.
pub(crate) fn python_executable() -> PathBuf {
    if let Some(value) = env::var_os(PYTHON_VARIABLE) {
        return PathBuf::from(value);
    }
    let sibling: Option<PathBuf> = match env::current_exe() {
        Ok(current) => current.parent().and_then(|directory| {
            SIBLING_NAMES
                .iter()
                .map(|name| directory.join(name))
                .find(|candidate| candidate.is_file())
        }),
        Err(_) => None,
    };
    sibling.unwrap_or_else(|| PathBuf::from(PATH_FALLBACK))
}
