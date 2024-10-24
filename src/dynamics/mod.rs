use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
pub mod ball;
mod centre;
pub mod collide;
pub mod maths;

use ball::{Ball, Container};
use collide::Collide;
use maths::FloatVec;

pub enum DynamicsError {
    StationaryCollision,
    PointParticleCollision,
    IntersectingParticles,
    SimulationFailure,
}

#[pymethods]
impl Ball {
    #[new]
    #[pyo3(signature = (pos=(0f64, 0f64), vel=(0f64, 0f64), r=0.01f64))]
    fn py_new(pos: (f64, f64), vel: (f64, f64), r: f64) -> Self {
        Self::new(pos.into(), vel.into(), r)
    }

    #[getter(pos)]
    fn py_get_pos(&self) -> (f64, f64) {
        (self.pos.x, self.pos.y)
    }

    #[setter(pos)]
    fn py_set_pos(&mut self, pos: (f64, f64)) {
        let (x, y) = pos;
        self.pos = FloatVec { x, y }
    }

    #[getter(vel)]
    fn py_get_vel(&self) -> (f64, f64) {
        (self.vel.x, self.vel.y)
    }

    #[setter(vel)]
    fn py_set_vel(&mut self, vel: (f64, f64)) {
        let (x, y) = vel;
        self.vel = FloatVec { x, y }
    }

    #[pyo3(signature = (t, delta=1e-6))]
    #[pyo3(name = "step")]
    fn py_step(&mut self, t: f64, delta: f64) {
        self.pos += self.vel * t * (1. - delta);
    }

    #[pyo3(name = "time_to_collision")]
    fn py_time_to_collision(&self, other: Py<Ball>) -> Option<f64> {
        Python::with_gil(|py| {
            let other = other.borrow(py);
            self.time_to_collision(&*other)
        })
    }

    #[pyo3(name = "time_to_container_collision")]
    fn py_time_to_container_collision(&self, other: Py<Container>) -> Option<f64> {
        Python::with_gil(|py| {
            let other = other.borrow(py);
            self.time_to_collision(&*other)
        })
    }

    #[pyo3(name = "collide")]
    fn py_collide(&mut self, other: Py<Ball>) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut other = other.borrow_mut(py);
            self.collide(&mut *other)
                .map_err(|_| PyValueError::new_err("Problem in the collision."))
        })
    }

    #[pyo3(name = "container_collide")]
    fn py_container_colllide(&mut self, container: Py<Container>) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut container = container.borrow_mut(py);
            self.collide(&mut *container)
                .map_err(|_| PyValueError::new_err("Problem in the collision."))
        })
    }

    fn pair_hash(&self, other: Py<Ball>) -> f64 {
        Python::with_gil(|py| {
            let other = other.borrow(py);
            let sum = self.vel + other.vel;
            let unit = FloatVec::new(1., 0.);
            sum.dot(&unit)
        })
    }

    fn v_squared(&self) -> f64 {
        self.vel.dot(&self.vel)
    }
}

#[pymethods]
impl Container {
    #[new]
    #[pyo3(signature = (r=1f64))]
    fn py_new(r: f64) -> Self {
        // Here, we don't need to put anything in the `pressure_tx` as
        // `Container`s constructed in Python don't need to send anything.
        Self::new(r, None)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn time_to_collision() {
        let b1 = ball::Ball::new((0., 0.).into(), (1., 0.).into(), 0.1);
        let b2 = ball::Ball::new((1., 0.).into(), (0., 0.).into(), 0.1);
        let ttc = b1.time_to_collision(&b2).unwrap();
        println!("{}", ttc);

        assert!(maths::approx_eq_f64(ttc, 0.8, 1));
    }
}
