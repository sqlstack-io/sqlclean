//! `sqlclean skills`: install the bundled SQLClean skill for AI coding agents.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use include_dir::{Dir, DirEntry, include_dir};

use crate::command::constants::{
    DEFAULT_SKILL_TARGETS, EXIT_SUCCESS, SKILL_FILE_EXTENSION, SKILL_FILE_SUFFIX,
    SKILL_GENERATED_MARKER, SKILL_NAME, SKILL_TARGETS,
};
use crate::command::models::SkillsArgs;

static SKILL: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/skill");

pub(crate) fn run(root: &Path, arguments: &SkillsArgs) -> Result<u8, String> {
    let base: PathBuf = if arguments.global {
        home()?
    } else {
        root.to_path_buf()
    };
    let targets: Vec<&str> = if arguments.target.is_empty() {
        DEFAULT_SKILL_TARGETS.to_vec()
    } else {
        arguments.target.iter().map(String::as_str).collect()
    };
    let mut skill_dirs: Vec<PathBuf> = Vec::new();
    for target in targets {
        let (_, local, global) = SKILL_TARGETS
            .iter()
            .find(|(name, _, _)| *name == target)
            .ok_or_else(|| {
                format!("unknown skill target '{target}'; expected agents, claude or opencode")
            })?;
        let relative: &str = if arguments.global { global } else { local };
        let skill_dir: PathBuf = base.join(relative).join(SKILL_NAME);
        if !skill_dirs.contains(&skill_dir) {
            skill_dirs.push(skill_dir);
        }
    }
    let files: Vec<(String, &str)> = packaged_files(&SKILL);
    if !arguments.force {
        for skill_dir in &skill_dirs {
            refuse_unowned(skill_dir, &files)?;
        }
    }
    let mut stdout = std::io::stdout();
    for skill_dir in &skill_dirs {
        install(skill_dir, &files)?;
        let _ = writeln!(stdout, "Installed {}", skill_dir.join("SKILL.md").display());
    }
    Ok(EXIT_SUCCESS)
}

fn home() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| "cannot find the home directory for --global".to_owned())
}

fn packaged_files(dir: &'static Dir<'static>) -> Vec<(String, &'static str)> {
    let mut files: Vec<(String, &str)> = Vec::new();
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(child) => files.extend(packaged_files(child)),
            DirEntry::File(file) => {
                let relative: String = file.path().to_string_lossy().replace('\\', "/");
                if let (true, Some(text)) =
                    (relative.ends_with(SKILL_FILE_SUFFIX), file.contents_utf8())
                {
                    files.push((relative, text));
                }
            }
        }
    }
    files.sort();
    files
}

fn generated(path: &Path) -> bool {
    fs::read_to_string(path).is_ok_and(|text| text.contains(SKILL_GENERATED_MARKER))
}

fn refuse_unowned(skill_dir: &Path, files: &[(String, &str)]) -> Result<(), String> {
    for (relative, _) in files {
        let path: PathBuf = skill_dir.join(relative);
        if path.is_file() && !generated(&path) {
            return Err(format!(
                "refusing to overwrite a skill file SQLClean did not generate: {}; rerun with --force to replace it",
                path.display()
            ));
        }
    }
    Ok(())
}

/// Write every packaged file, then remove generated files the package no longer ships.
fn install(skill_dir: &Path, files: &[(String, &str)]) -> Result<(), String> {
    for (relative, text) in files {
        let path: PathBuf = skill_dir.join(relative);
        if fs::read_to_string(&path).is_ok_and(|current| current == *text) {
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(&path, text).map_err(|error| format!("{}: {error}", path.display()))?;
    }
    for path in markdown_files(skill_dir) {
        let relative: String = path
            .strip_prefix(skill_dir)
            .map(|relative| relative.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        if !files.iter().any(|(shipped, _)| *shipped == relative) && generated(&path) {
            fs::remove_file(&path).map_err(|error| error.to_string())?;
            if let Some(parent) = path.parent() {
                let _ = fs::remove_dir(parent);
            }
        }
    }
    Ok(())
}

fn markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path: PathBuf = entry.path();
        if path.is_dir() {
            found.extend(markdown_files(&path));
        } else if path
            .extension()
            .is_some_and(|extension| extension == SKILL_FILE_EXTENSION)
        {
            found.push(path);
        }
    }
    found
}
