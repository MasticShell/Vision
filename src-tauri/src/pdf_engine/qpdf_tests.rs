use crate::pdf_engine::qpdf;

/// Verify that `resolve_qpdf_standalone` returns a non-empty path in all
/// environments (it always returns at least the bare exe name as a fallback).
#[test]
fn resolve_qpdf_standalone_always_returns_a_path() {
    let path = qpdf::resolve_qpdf_standalone();
    assert!(
        !path.as_os_str().is_empty(),
        "resolve_qpdf_standalone must return a non-empty PathBuf"
    );
    assert!(
        path.file_name().is_some(),
        "resolve_qpdf_standalone must return a path with a filename component"
    );
}

/// Verify that the resolved qpdf binary is executable and reports a version.
///
/// This test is CONDITIONALLY SKIPPED if qpdf is not present in the
/// environment (CI without apt-get install qpdf, or offline dev machines).
/// It never panics on absence — it only fails if qpdf is present but broken.
#[test]
fn qpdf_binary_runs_and_reports_version() {
    let qpdf_bin = qpdf::resolve_qpdf_standalone();

    // Probe: can we even spawn qpdf?
    let probe = std::process::Command::new(&qpdf_bin)
        .arg("--version")
        .output();

    let output = match probe {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // qpdf is not installed in this environment — skip gracefully.
            eprintln!(
                "SKIP: qpdf not found at {:?} (install qpdf to run this test)",
                qpdf_bin
            );
            return;
        }
        Err(e) => panic!("qpdf spawn failed unexpectedly: {e}"),
        Ok(out) => out,
    };

    assert!(
        output.status.success(),
        "qpdf --version exited with non-zero status: {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.to_lowercase().contains("qpdf"),
        "qpdf --version output should contain 'qpdf', got: {combined}"
    );
}
