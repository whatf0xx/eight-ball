use pyo3::prelude::*;
use simulation::Simulation;
mod dynamics;
mod simulation;
use dynamics::dynamics::Ball;

#[pyfunction]
fn test() {
    println!("This executed in Rust!");
}

#[pymodule]
fn eight_ball(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(test, m)?)?;
    Ok(())
}
