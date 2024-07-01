use pyo3::prelude::*;
mod dynamics;
use dynamics::ball::Ball;
mod simulation;
use simulation::simulate::Simulation;

#[pymodule]
fn eight_ball(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Ball>()?;
    m.add_class::<Simulation>()?;
    Ok(())
}
