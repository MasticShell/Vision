//! OCR: make a scanned PDF searchable.
//!
//! M1 status: the Poppler rasterisation step (pdftoppm) has been stubbed out
//! and returns `CapabilityUnavailable`. Tesseract availability probing and
//! language listing still work — only `ocr()` itself is gated. The full
//! pipeline (rasterise with PDFium → Tesseract → merge with qpdf) is
//! scheduled for M2.

use crate::error::AppError;
use crate::models::{JobHandle, PagePick};
use crate::pdf_engine::ocr_langs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use tauri::Manager;

fn tesseract_exe() -> &'static str {
    "tesseract"
}

/// Locate the Tesseract binary.
pub fn resolve_tesseract(app: &tauri::AppHandle) -> PathBuf {
    let exe = tesseract_exe();

    if let Some(found) = find_bundled_tesseract(app, exe) {
        return found;
    }

    for c in ["/usr/bin/tesseract", "/usr/local/bin/tesseract"] {
        let p = PathBuf::from(c);
        if p.exists() {
            return p;
        }
    }
    PathBuf::from(exe)
}

/// Whether the OCR pipeline is available.
///
/// In M1, OCR capability is explicitly unavailable (`false`) because Poppler
/// rasterisation was removed and PDFium integration is scheduled for M2.
/// Even if Tesseract is installed on the host, the complete OCR pipeline
/// cannot function without a page rasteriser.
pub fn available(_app: &tauri::AppHandle) -> bool {
    false
}

fn app_roots(app: &tauri::AppHandle) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(res) = app.path().resource_dir() {
        roots.push(res);
    }
    if let Ok(cur) = std::env::current_exe() {
        if let Some(parent) = cur.parent() {
            roots.push(parent.to_path_buf());
        }
    }
    roots
}

fn find_bundled_tesseract(app: &tauri::AppHandle, exe: &str) -> Option<PathBuf> {
    for root in app_roots(app) {
        for candidate in [
            root.join("tesseract").join(exe),
            root.join("resources").join("tesseract").join(exe),
            root.join("binaries").join(exe),
            root.join("resources").join("binaries").join(exe),
        ] {
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

fn tessdata_dir_for_tool(exe: &Path) -> Option<PathBuf> {
    let bin_dir = exe.parent()?;
    for candidate in [
        bin_dir.join("tessdata"),
        bin_dir.join("..").join("tessdata"),
    ] {
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn configure_tesseract_command(cmd: &mut Command, exe: &Path) {
    let Some(bin_dir) = exe.parent().filter(|p| !p.as_os_str().is_empty()) else {
        return;
    };

    cmd.current_dir(bin_dir);

    if let Some(current_path) = std::env::var_os("PATH") {
        let mut paths = std::env::split_paths(&current_path).collect::<Vec<_>>();
        paths.insert(0, bin_dir.to_path_buf());
        if let Ok(joined) = std::env::join_paths(paths) {
            cmd.env("PATH", joined);
        }
    } else {
        cmd.env("PATH", bin_dir);
    }

    if let Some(tessdata) = tessdata_dir_for_tool(exe) {
        cmd.env("TESSDATA_PREFIX", &tessdata);
    }
}

fn missing() -> AppError {
    AppError::new(
        "OCR_MISSING",
        "Tesseract not found",
        "OCR needs Tesseract, which couldn't be located.",
    )
    .with_suggestion("Install Tesseract — e.g. sudo apt install tesseract-ocr.")
}

fn langs_failed(details: impl Into<String>) -> AppError {
    AppError::new(
        "OCR_LANGS_FAILED",
        "Could not list OCR languages",
        "Tesseract did not report any installed language packs.",
    )
    .with_details(details.into())
    .with_suggestion(
        "Install Tesseract language packs locally (e.g. sudo apt install tesseract-ocr-eng). Vision does not download them.",
    )
}

/// Installed Tesseract language codes from the same binary and tessdata the job uses.
pub fn list_langs(app: &tauri::AppHandle) -> Result<Vec<String>, AppError> {
    let exe = resolve_tesseract(app);
    let mut cmd = Command::new(&exe);
    configure_tesseract_command(&mut cmd, &exe);
    if let Some(tessdata) = tessdata_dir_for_tool(&exe) {
        cmd.arg("--tessdata-dir").arg(tessdata);
    }
    cmd.arg("--list-langs");
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let out = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            missing()
        } else {
            langs_failed(e.to_string())
        }
    })?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}\n{stderr}");

    if !out.status.success() {
        return Err(langs_failed(combined.trim()));
    }

    let langs = ocr_langs::parse_tesseract_list_langs(&combined);
    if langs.is_empty() {
        return Err(langs_failed(combined.trim()));
    }
    Ok(langs)
}

/// OCR every page of the combined document into one searchable PDF.
///
/// **M1 stub**: The rasterisation step previously used Poppler's `pdftoppm`.
/// That dependency has been removed. This function now unconditionally returns
/// `CapabilityUnavailable`. The full pipeline (PDFium rasterise → Tesseract
/// → qpdf merge) is planned for M2.
pub fn ocr(
    _app: &tauri::AppHandle,
    _handle: &Arc<JobHandle>,
    _job_id: &str,
    _picks: &[PagePick],
    _output: &str,
    _lang: &str,
) -> Result<Vec<String>, AppError> {
    Err(AppError::new(
        "CapabilityUnavailable",
        "OCR temporarily unavailable",
        "OCR has been disabled during the M1 architecture migration. \
         The pipeline that previously relied on Poppler (pdftoppm) for \
         rasterisation has been removed. A PDFium-based replacement is \
         planned for M2.",
    ))
}
