use crate::dynamics::ball::Ball;
use pyo3::{exceptions::PyValueError, prelude::*};
mod data;
mod event;
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

    /// Run through `n` collisions, usually to thermalise the simulation.
    #[pyo3(name = "run_n_collisions")]
    fn py_run_n_collisions(&mut self, n: usize) -> PyResult<()> {
        println!("Running through collisions...");
        for _ in tqdm(0..n) {
            self.py_next_collision()?;
        }
        Ok(())
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

        println!("Calculating collisions...");
        for _ in tqdm(0..no_collisions) {
            self.py_next_collision()?;
            let collision_delta_t = self.global_time - current_time;
            current_time = self.global_time;
            tx_raw.send(collision_delta_t).unwrap();
        }
        // drop the tx_raw to cause the channel to hang up
        drop(tx_raw);

        let (tx_hist, rx_hist) = mpsc::channel();
        thread::spawn(move || {
            let data = Box::new(rx_raw.into_iter());
            let hist = Histogram::bin(left, right, bins, data);
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

    /// Run the simulation and track the positions of the balls. Panic in the
    /// secondary thread when a ball ends up outside the container and give the
    /// collision number and the global time at which it happened.
    fn track_positions(&mut self, no_collisions: usize) -> PyResult<()> {
        let (tx, rx) = mpsc::channel();

        println!("Calculating collisions...");
        for i in tqdm(0..no_collisions) {
            self.py_next_collision()?;
            let r_squareds: Vec<f64> = self
                .balls
                .iter()
                .map(|ball| ball.pos().dot(ball.pos()))
                .collect();
            let global_time = self.global_time;
            tx.send((i, global_time, r_squareds)).unwrap();
        }
        // drop the tx_raw to cause the channel to hang up
        drop(tx);

        let checker = thread::spawn(move || {
            for info in rx {
                let (i, global_time, r_squareds) = info;
                for r_squared in r_squareds {
                    if r_squared > 1f64 {
                        panic!("Collision: {};\tGlobal time: {}", i, global_time);
                    }
                }
            }
        });

        checker.join().unwrap();

        Ok(())
    }

    /// Run the simulation and record the times between which `n` collisions
    /// take place. Aggregate the data into a histogram.  The system must have
    /// previously been initialised, otherwise this is undefined.
    fn nth_collision_times(
        &mut self,
        n: usize,
        no_collisions: usize,
        left: f64,
        right: f64,
        bins: usize,
    ) -> PyResult<HashMap<String, PyObject>> {
        let (tx_raw, rx_raw) = mpsc::channel();
        let mut current_time = 0f64;

        println!("Calculating collisions...");
        for _ in tqdm(0..no_collisions / n) {
            let mut local_sum = 0f64;
            for _ in 0..n {
                self.py_next_collision()?;
                let collision_delta_t = self.global_time - current_time;
                current_time = self.global_time;
                local_sum += collision_delta_t;
            }
            tx_raw.send(local_sum).unwrap();
        }
        // drop the tx_raw to cause the channel to hang up
        drop(tx_raw);

        let (tx_hist, rx_hist) = mpsc::channel();
        thread::spawn(move || {
            let data = Box::new(rx_raw.into_iter());
            let hist = Histogram::bin(left, right, bins, data);
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
