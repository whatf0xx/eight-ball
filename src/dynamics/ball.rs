use crate::dynamics::maths::{approx_eq_f64, FloatVec};
use pyo3::prelude::*;

#[pyclass(subclass)]
#[pyo3(name = "_Ball")]
#[derive(Clone, Default)]
pub struct Ball {
    pub(crate) pos: FloatVec,
    pub(crate) vel: FloatVec,
    #[pyo3(get, set)]
    pub(crate) r: f64,
}

impl Ball {
    pub fn new(pos: FloatVec, vel: FloatVec, r: f64) -> Ball {
        Ball { pos, vel, r }
    }

    pub fn pos(&self) -> &FloatVec {
        &self.pos
    }

    pub fn vel(&self) -> &FloatVec {
        &self.vel
    }

    pub fn set_vel(&mut self, new_vel: FloatVec) {
        self.vel = new_vel
    }

    pub fn step(&mut self, t: f64) {
        self.pos += self.vel * t
    }

    pub fn com_velocity(a: &Ball, b: &Ball) -> FloatVec {
        let vel_a = a.vel;
        let vel_b = b.vel;

        (vel_a + vel_b) * 0.5
    }

    pub fn touching(&self, other: &Ball) -> bool {
        // returns true if the Balls are touching
        let centres_distance_squared = (self.r + other.r) * (self.r + other.r);
        let relative_displacement = self.pos - other.pos;
        let distance_squared = relative_displacement.dot(&relative_displacement);

        approx_eq_f64(centres_distance_squared, distance_squared, 1)
    }
}

pub struct Container {
    pub(crate) r: f64,
}
