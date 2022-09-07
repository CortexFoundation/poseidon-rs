extern crate ff;
extern crate num_bigint;

#[macro_use]
extern crate lazy_static;

use pyo3::prelude::*;

pub mod poseidon;
pub mod babyjubjub;

#[pymodule]
fn crypto_rs(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(
            poseidon::poseidon_params, m)?)?;
    m.add_function(wrap_pyfunction!(
            poseidon::poseidon_hash, m)?)?;
    m.add_function(wrap_pyfunction!(
            poseidon::multi_poseidon_hash, m)?)?;

    m.add_function(wrap_pyfunction!(
            babyjubjub::eddsa_verify, m)?)?;

    // Python::with_gil(|py| -> PyResult<()> {
    //     let fast = PyModule::new(py, "fast")?;
    //     fast.add_function(wrap_pyfunction!(fast::poseidon_params, m)?)?;
    //     fast.add_function(wrap_pyfunction!(fast::poseidon_hash, m)?)?;
    //     m.add_submodule(fast)?;
    //     Ok(())
    // })?;

    Ok(())
}
