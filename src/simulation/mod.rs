use crate::dynamics::ball::Ball;
use pyo3::{exceptions::PyValueError, prelude::*};
mod collision;
mod histogram;
pub mod simulate;
use simulate::Simulation;

#[pymethods]
impl Simulation {
    #[new]
    fn py_new(radius: f64) -> Simulation {
        Self::new(radius)
    }

    fn add_balls(&mut self, balls: Vec<Py<Ball>>) {
        Python::with_gil(|py| {
            for ball in balls {
                let ball = ball.borrow(py).to_owned();
                self.balls.push(ball);
            }
        })
    }

    fn get_balls(&self) -> Vec<Ball> {
        let mut out = Vec::new();
        for ball in self.balls.iter() {
            out.push(ball.clone())
        }
        out
    }

    fn initialise(&mut self) {
        // Based on the balls added to the container, initialise
        // the dynamics of the `Simulation` so that the collision
        // queue represents the correct dynamics.
        self.generate_collision_queue();
        self.generate_container_collisions();
    }

    #[pyo3(name = "next_collision")]
    fn py_next_collision(&mut self) -> PyResult<()> {
        self.step_through_collision()
            .map_err(|_| PyValueError::new_err("Bad dynamics in the simulation."))
    }
}
