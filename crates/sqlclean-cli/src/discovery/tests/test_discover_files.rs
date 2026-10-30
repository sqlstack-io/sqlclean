use std::fs;
use std::path::PathBuf;

use crate::config::main::load_config::load_config;
use crate::discovery::main::discover_files::discover_files;
use crate::discovery::tests::test_types::DiscoveryTestCase;

const PROJECT: [(&str, &str); 7] = [
    (
        "sqlclean.toml",
        "dialect = \"duckdb\"\nexclude = [\"vendor/**\"]\n[[dialects]]\npaths = [\"warehouse/**\"]\ndialect = \"snowflake\"\n",
    ),
    (".gitignore", "build/\n"),
    ("models/orders.sql", "SELECT 1"),
    ("warehouse/customers.sql", "SELECT 2"),
    ("vendor/third_party.sql", "SELECT 3"),
    ("build/generated.sql", "SELECT 4"),
    ("notes.txt", "not sql"),
];

#[test]
fn given_patterns_and_ignore_files_when_discovering_then_selects_files_and_dialects()
-> Result<(), String> {
    let test_cases = [
        DiscoveryTestCase {
            description: "a full walk skips excluded and git-ignored files and maps dialects by path",
            explicit_paths: &[],
            expected_files: &[
                ("models/orders.sql", "duckdb"),
                ("warehouse/customers.sql", "snowflake"),
            ],
        },
        DiscoveryTestCase {
            description: "an explicit file is read on its own",
            explicit_paths: &["models/orders.sql"],
            expected_files: &[("models/orders.sql", "duckdb")],
        },
        DiscoveryTestCase {
            description: "an explicit directory is walked with the same filters",
            explicit_paths: &["warehouse"],
            expected_files: &[("warehouse/customers.sql", "snowflake")],
        },
    ];
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    let root = directory.path();
    for (path, text) in PROJECT {
        let file = root.join(path);
        fs::create_dir_all(file.parent().unwrap_or(root)).map_err(|error| error.to_string())?;
        fs::write(file, text).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(root.join(".git")).map_err(|error| error.to_string())?;
    let config = load_config(None, root)?;
    for test_case in &test_cases {
        let paths: Vec<PathBuf> = test_case
            .explicit_paths
            .iter()
            .map(|path| root.join(path))
            .collect();
        let files = discover_files(&config, &paths)?;
        let found: Vec<(&str, &str)> = files
            .iter()
            .map(|file| (file.path.as_str(), file.dialect.as_str()))
            .collect();
        assert_eq!(found, test_case.expected_files, "{}", test_case.description);
    }
    Ok(())
}
