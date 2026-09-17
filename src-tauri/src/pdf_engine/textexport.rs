//! Export a PDF's text to a .txt file.
//!
//! M1 status: Text export via `pdftotext` has been disabled during the M1
//! architecture migration. Returns `CapabilityUnavailable`. PDFium-based text
//! extraction will be introduced in M2.

use crate::error::AppError;
use crate::models::JobHandle;
use std::sync::Arc;

pub fn export_text(
    _app: &tauri::AppHandle,
    _handle: &Arc<JobHandle>,
    _job_id: &str,
    _input: &str,
    _output: &str,
    _first: Option<u32>,
    _last: Option<u32>,
) -> Result<Vec<String>, AppError> {
    Err(AppError::new(
        "CapabilityUnavailable",
        "Feature temporarily unavailable",
        "Text export is disabled during the M1 architecture migration.",
    ))
}
