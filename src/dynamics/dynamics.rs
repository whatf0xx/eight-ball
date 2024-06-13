use crate::dynamics::maths::{approx_eq_f64, FloatVec};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

pub enum DynamicsError {
    StationaryCollision,
    PointParticleCollision,
    IntersectingParticles,
}

#[pyclass(subclass)]
#[pyo3(name = "_Ball")]
#[derive(Clone, Default)]
pub struct Ball {
    pos: FloatVec,
    vel: FloatVec,
    #[pyo3(get, set)]
    r: f64,
}

#[pymethods]
impl Ball {
    #[new]
    #[pyo3(signature = (pos=(0f64, 0f64), vel=(0f64, 0f64), r=1f64))]
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
            self.time_to_collision(&other)
        })
    }

    #[pyo3(name = "collide")]
    fn py_collide(&mut self, other: Py<Ball>) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut other = other.borrow_mut(py);
            self.collide(&mut other)
                .map_err(|_| PyValueError::new_err("Problem in the collision."))
        })
    }

    #[pyo3(name = "container_collide")]
    fn py_container_colllide(&mut self, container: Py<Ball>) -> PyResult<()> {
        Python::with_gil(|py| {
            let container = container.borrow(py);
            self.container_collide(&container)
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

    pub fn time_to_collision(&self, other: &Ball) -> Option<f64> {
        let dr = self.pos - other.pos;
        let dv = self.vel - other.vel;
        let dv_squared = dv.dot(&dv);

        let lhs = dv_squared * (self.r + other.r) * (self.r + other.r);
        let rhs = dr.cross_squared(&dv);

        if lhs < rhs {
            // equivalent to asking if discriminant < 0
            None
        } else {
            // find the smallest positive solution
            let disc = lhs - rhs;
            let r1 = -(dv.dot(&dr) + disc.sqrt()) / dv_squared;
            let r2 = -(dv.dot(&dr) - disc.sqrt()) / dv_squared;

            let (r_min, r_max) = (r1.min(r2), r1.max(r2));
            if r_min.is_sign_positive() {
                Some(r_min)
            } else if r_max.is_sign_positive() {
                Some(r_max)
            } else {
                // If time to collision is 0, we also say that they don't collide
                // because the only way this happens in elastic dynamics is if they
                // were already stationary relative to each other.
                None
            }
        }
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

    pub fn normalised_difference(&self, other: &Ball) -> Result<FloatVec, DynamicsError> {
        // calculate the normalised vector that points from the centre of self to other. If no
        // such vector exists, return a DynamicsError that reflects this.
        if self.pos.approx_eq(&other.pos, 1) {
            // if the Balls are on top of each other, it is impossible to draw a normaliseable
            // line between the two centres.
            Err(DynamicsError::IntersectingParticles)
        } else {
            let diff = other.pos - self.pos;
            let diff_squared = diff.dot(&diff);
            Ok(diff / diff_squared.sqrt())
        }
    }

    pub fn collide(&mut self, other: &mut Ball) -> Result<(), DynamicsError> {
        // Calculate and update the trajectories following a collision for `self` and `other`.
        // If this would generate an arithmetic error, for example because the balls were
        // relatively stationary, or both have zero volume, then an `Err()` with appropriate
        // information is returned, instead.

        // The calculation is performed using the 'line of centers' method: assuming an elastic collision,
        // momentum along the vector that is tangential to the point of collision on both balls is conserved,
        // as the normal force exerted by each ball is strictly perpendicular to this. In the direction of
        // the balls' normal, the velocities are swapped.

        let normed_normal = self.normalised_difference(other)?;
        let loc = normed_normal.anti_clockwise_perpendicular();

        let alpha_1 = self.vel.dot(&loc);
        let beta_1 = self.vel.dot(&normed_normal);

        let alpha_2 = other.vel.dot(&loc);
        let beta_2 = other.vel.dot(&normed_normal);

        self.set_vel(alpha_1 * loc + beta_2 * normed_normal);
        other.set_vel(alpha_2 * loc + beta_1 * normed_normal);
        Ok(())
    }

    pub fn container_collide(&mut self, container: &Ball) -> Result<(), DynamicsError> {
        // Calculate and update the trajectory for a `Ball` colliding with a container, i.e.
        // a stationary `Ball` which we also assume totally contains `self`.

        let normed_normal = self.normalised_difference(container)?;
        let loc = normed_normal.anti_clockwise_perpendicular();

        let alpha = self.vel.dot(&loc);
        let beta = self.vel.dot(&normed_normal);

        self.set_vel(alpha * loc - beta * normed_normal);
        Ok(())
    }
}
