use pyo3::prelude::*;
use simulation::Simulation;
mod dynamics;
mod simulation;
use dynamics::dynamics::Ball;

#[pymodule]
fn eight_ball(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Ball>()?;
    m.add_class::<Simulation>()?;
    Ok(())
}
