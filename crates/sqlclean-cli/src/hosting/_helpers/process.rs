//! Start the host, write the request to its standard input and parse its standard output.

use std::io::Write;
use std::process::{Command, Stdio};

use crate::hosting::_helpers::interpreter::python_executable;
use crate::hosting::constants::{HOST_PROGRAM, MISSING_PACKAGE_HELP};
use crate::hosting::models::{HostRequest, HostResponse};

const MISSING_MODULE_MARKER: &str = "No module named 'sqlclean";

pub(crate) fn run_host(request: &HostRequest) -> Result<HostResponse, String> {
    let payload: Vec<u8> = serde_json::to_vec(request).map_err(|error| error.to_string())?;
    let python = python_executable();
    let mut child = Command::new(&python)
        .args(["-c", HOST_PROGRAM])
        .current_dir(&request.root)
        .env("PYTHONHASHSEED", "0")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            format!(
                "could not start {}: {error}. {MISSING_PACKAGE_HELP}",
                python.display()
            )
        })?;
    if let Some(mut input) = child.stdin.take() {
        input
            .write_all(&payload)
            .map_err(|error| format!("could not send custom Rules request: {error}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| format!("custom Rules host failed: {error}"))?;
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    if stderr.contains(MISSING_MODULE_MARKER) {
        return Err(MISSING_PACKAGE_HELP.to_owned());
    }
    let response: HostResponse = serde_json::from_slice(&output.stdout).map_err(|error| {
        format!(
            "custom Rules host returned no result ({error}):\n{}",
            stderr.trim_end()
        )
    })?;
    match response.error {
        Some(error) => Err(error),
        None if !output.status.success() => Err(format!(
            "custom Rules host exited with {}:\n{}",
            output.status,
            stderr.trim_end()
        )),
        None => Ok(response),
    }
}
