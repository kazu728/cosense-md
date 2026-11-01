use pyo3::prelude::*;

#[pyfunction]
fn convert(text: &str) -> String {
    cosense_core::convert(text)
}

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
