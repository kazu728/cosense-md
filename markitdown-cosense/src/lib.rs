//! PyO3 binding: the compiled `markitdown_cosense._core` module. It is a thin
//! shim — all conversion logic lives in `cosense-core`.

use pyo3::prelude::*;

/// Convert Cosense text to Markdown.
#[pyfunction]
fn convert(text: &str) -> String {
    cosense_core::convert(text)
}

/// Detection heuristic backing the plugin's `accepts` check.
#[pyfunction]
fn looks_like_cosense(text: &str) -> bool {
    cosense_core::looks_like_cosense(text)
}

#[pymodule]
fn _core(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(convert, module)?)?;
    module.add_function(wrap_pyfunction!(looks_like_cosense, module)?)?;
    Ok(())
}
