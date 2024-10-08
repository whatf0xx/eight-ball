use crate::dynamics::ball::Ball;
use pyo3::{exceptions::PyValueError, prelude::*};
mod collision;
mod histogram;
use histogram::Histogram;
pub mod simulate;
use simulate::Simulation;
use std::{collections::HashMap, sync::mpsc, thread};
use tqdm::tqdm;

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

    /// Run the simulation and record the times at which collisions take place,
    /// aggregating them into a histogram which is returned in the form of a
    /// Python dictionary that maps the bin centres to the counts. The system
    /// must have previously been initialised, otherwise this is undefined.
    fn collision_times(
        &mut self,
        no_collisions: usize,
        left: f64,
        right: f64,
        bins: usize,
    ) -> PyResult<HashMap<String, PyObject>> {
        let (tx_raw, rx_raw) = mpsc::channel();
        let mut current_time = 0f64;

        for _ in tqdm(0..no_collisions) {
            self.py_next_collision()?;
            let collision_delta_t = self.global_time - current_time;
            current_time = self.global_time;
            tx_raw.send(collision_delta_t).unwrap();
        }

        let (tx_hist, rx_hist) = mpsc::channel();
        thread::spawn(move || {
            let data = Box::new(rx_raw.into_iter());
            let hist = Histogram::bin(left, right, bins, data);
            println!("Actually returned from the binning function!");
            tx_hist.send(hist).unwrap();
        });

        let hist = rx_hist.recv().unwrap();
        let dict_elements = Python::with_gil(|py| {
            vec![
                (String::from("width"), hist.width().to_object(py)),
                (String::from("centres"), hist.centres().to_object(py)),
                (String::from("counts"), hist.counts().to_object(py)),
            ]
        });
        let dict_map: HashMap<String, PyObject> = dict_elements.into_iter().collect();
        Ok(dict_map)
    }
}
