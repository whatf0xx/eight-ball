use crate::dynamics::ball::{Ball, Container};
use crate::dynamics::maths::FloatVec;
use crate::dynamics::DynamicsError;

pub trait Centre {
    fn get_centre(&self) -> FloatVec;
}

impl Centre for Ball {
    fn get_centre(&self) -> FloatVec {
        self.pos
    }
}

impl Centre for Container {
    fn get_centre(&self) -> FloatVec {
        FloatVec::origin()
    }
}

pub fn normalised_difference<T, U>(a: &T, b: &U) -> Result<FloatVec, DynamicsError>
where
    T: Centre,
    U: Centre,
{
    // calculate the normalised vector that points from the centre of self to other. If no
    // such vector exists, return a DynamicsError that reflects this.
    let centre_a = a.get_centre();
    let centre_b = b.get_centre();
    if centre_a.approx_eq(&centre_b, 1) {
        // if the Balls are on top of each other, it is impossible to draw a normaliseable
        // line between the two centres.
        Err(DynamicsError::IntersectingParticles)
    } else {
        let diff = centre_a - centre_b;
        let diff_squared = diff.dot(&diff);
        Ok(diff / diff_squared.sqrt())
    }
}
