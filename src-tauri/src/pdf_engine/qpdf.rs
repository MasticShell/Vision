//! qpdf binary resolution and small qpdf-backed queries.

use crate::error::AppError;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::Manager;

/// Platform binary name for qpdf.
fn exe_name() -> &'static str {
    "qpdf"
}

/// Locate qpdf without a Tauri handle (Edit PDF `--check`, tests).
pub fn resolve_qpdf_standalone() -> PathBuf {
    let exe = exe_name();

    // Bundled next to the executable.
    if let Ok(cur) = std::env::current_exe() {
        if let Some(parent) = cur.parent() {
            let candidate = parent.join("binaries").join(exe);
            if candidate.exists() {
                return candidate;
            }
        }
    }

    // Common absolute install locations. A Finder-launched .app does NOT
    // inherit the shell PATH (so Homebrew/MacPorts dirs are missing), so we
    // probe them explicitly before relying on PATH.
    for candidate in [
        "/usr/bin/qpdf",          // Linux distro packages
        "/usr/local/bin/qpdf",    // common Linux local
    ] {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return p;
        }
    }

    // Fall back to PATH (works when launched from a terminal / dev).
    PathBuf::from(exe)
}

/// Locate the qpdf binary. Prefers a bundled copy under `binaries/`, falling
/// back to the system PATH (by returning the bare exe name).
pub fn resolve_qpdf(app: &tauri::AppHandle) -> PathBuf {
    let exe = exe_name();

    // 1. Bundled next to app resources.
    if let Ok(res) = app.path().resource_dir() {
        let candidate = res.join("binaries").join(exe);
        if candidate.exists() {
            return candidate;
        }
    }

    resolve_qpdf_standalone()
}

/// Return the number of pages in `input` via `qpdf --show-npages`.
/// Any failure (spawn, non-zero exit, unparseable output) -> `invalid_pdf`.
pub fn npages(app: &tauri::AppHandle, input: &str) -> Result<u32, AppError> {
    let exe = resolve_qpdf(app);
    let mut cmd = Command::new(exe);
    cmd.arg("--show-npages");
    cmd.arg(input);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());



    let output = cmd.output().map_err(|_| AppError::invalid_pdf(input))?;
    if !output.status.success() && output.status.code() != Some(3) {
        return Err(AppError::invalid_pdf(input));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    text.trim()
        .parse::<u32>()
        .map_err(|_| AppError::invalid_pdf(input))
}

#[cfg(test)]
#[path = "qpdf_tests.rs"]
mod tests;
