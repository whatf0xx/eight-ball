use pyo3::prelude::*;
use simulation::Simulation;
mod dynamics;
mod simulation;

// #[pyclass]
// pub struct EBSim {
//     // A simulation of billard-ball particles interacting to model gas physics.
//     pub no_of_balls: usize,
//     simulation: Simulation,
// }

// impl EBSim {
//     #[new]
//     pub fn new(no_of_balls) -> Self {

//     }
// }
