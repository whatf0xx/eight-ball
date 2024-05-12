use crate::maths::{SafeFloat, SafeFloatError, SafeFloatVec};

#[derive(Clone)]
pub struct Ball {
    pos: SafeFloatVec,
    vel: SafeFloatVec,
    r: SafeFloat,
}

impl Ball {
    pub fn new(pos: SafeFloatVec, vel: SafeFloatVec, r: SafeFloat) -> Ball {
        Ball { pos, vel, r }
    }

    pub fn step(&mut self, t: SafeFloat) {
        self.pos = self.pos + self.vel * t;
    }

    pub fn pos(&self) -> &SafeFloatVec {
        &self.pos
    }

    pub fn vel(&self) -> &SafeFloatVec {
        &self.vel
    }

    pub fn set_vel(&mut self, new_vel: SafeFloatVec) {
        self.vel = new_vel
    }

    pub fn time_to_collision(&self, other: &Ball) -> Option<SafeFloat> {
        let dr = self.pos - other.pos;
        let dv = self.vel - other.vel;
        let dv_squared = dv.dot(&dv);

        let lhs = dv_squared * (self.r + other.r) * (self.r + other.r);
        let rhs = dr.cross_squared(&dv);

        if lhs < rhs {
            // equivalent to asking if discriminant < 0
            None
        } else if dv_squared.is_zero() {
            (-dv.dot(&dr) / dv_squared).ok()
        } else {
            // find the smallest positive solution
            let disc = lhs - rhs;
            let r1 = (-(dv.dot(&dr) + disc) / dv_squared).unwrap();
            let r2 = (-(dv.dot(&dr) - disc) / dv_squared).unwrap();

            let (r_min, r_max) = (r1.min(r2), r1.max(r2));
            if r_min.is_positive() {
                Some(r_min)
            } else if r_max.is_positive() {
                Some(r_max)
            } else {
                // If time to collision is 0, we also say that they don't collide
                // because the only way this happens in elastic dynamics is if they
                // were already stationary relative to each other.
                None
            }
        }
    }

    pub fn com_velocity(a: &Ball, b: &Ball) -> SafeFloatVec {
        let vel_a = a.vel;
        let vel_b = b.vel;

        (vel_a + vel_b) * SafeFloat::try_from(0.5).unwrap()
    }

    pub fn touching(&self, other: &Ball) -> bool {
        // returns true if the Balls are touching
        let centres_distance_squared = (self.r + other.r) * (self.r + other.r);
        let relative_displacement = self.pos - other.pos;
        let distance_squared = relative_displacement.dot(&relative_displacement);

        centres_distance_squared.approx_eq(&distance_squared, 1)
    }

    pub fn normalised_difference(&self, other: &Ball) -> Result<SafeFloatVec, SafeFloatError> {
        // calculate the normalised vector that points from the centre of self to other. If no
        // such vector exists, return a SafeFloatError that reflects this.
        if self.pos.approx_eq(&other.pos, 1) {
            // if the Balls are on top of each other, it is impossible to draw a normaliseable
            // line between the two centres.
            Err(SafeFloatError::BadVecError)
        } else {
            let diff = other.pos - self.pos;
            let diff_squared = diff.dot(&diff);
            Ok((diff / diff_squared.sqrt()).unwrap())
        }
    }

    pub fn calculate_collision(
        &self,
        other: &Ball,
    ) -> Result<(SafeFloatVec, SafeFloatVec), SafeFloatError> {
        // calculate the trajectories following a collision for `self` and `other` and return them as the `Ok()`
        // variant of a `Result`. If the collision calculation generates an arithmetic error, perhaps because the
        // balls were relatively stationary, or both have zero volume, then an `Err()` with appropriate
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

        Ok((
            (alpha_2 * loc + beta_1 * normed_normal),
            (alpha_1 * loc + beta_2 * normed_normal),
        ))
    }
}
