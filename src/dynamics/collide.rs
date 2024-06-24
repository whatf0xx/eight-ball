use crate::dynamics::ball::{Ball, Container};
use crate::dynamics::centre::normalised_difference;
use crate::dynamics::DynamicsError;

pub trait Collide<T> {
    fn time_to_collision(&self, other: &T) -> Option<f64>;
    fn collide(&mut self, other: &mut T) -> Result<(), DynamicsError>;
    // Calculate and update the trajectories following a collision for `self`
    // and `other`. If this would generate an arithmetic error, for example
    // because the particles were relatively stationary, or both have zero
    // volume, then an `Err()` with appropriate information is returned,
    // instead.
}

impl Collide<Ball> for Ball {
    fn time_to_collision(&self, other: &Ball) -> Option<f64> {
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

            smallest_positive(r1, r2)
        }
    }

    fn collide(&mut self, other: &mut Ball) -> Result<(), DynamicsError> {
        // The calculation is performed using the 'line of centers' method:
        // assuming an elastic collision, momentum along the vector that is
        // tangential to the point of collision on both balls is conserved,
        // as the normal force exerted by each ball is strictly perpendicular
        // to this. In the direction of the balls' normal, the velocities are
        // swapped.

        let normed_normal = normalised_difference(self, other)?;
        let loc = normed_normal.anti_clockwise_perpendicular();

        let alpha_1 = self.vel.dot(&loc);
        let beta_1 = self.vel.dot(&normed_normal);

        let alpha_2 = other.vel.dot(&loc);
        let beta_2 = other.vel.dot(&normed_normal);

        self.set_vel(alpha_1 * loc + beta_2 * normed_normal);
        other.set_vel(alpha_2 * loc + beta_1 * normed_normal);
        Ok(())
    }
}

impl Collide<Container> for Ball {
    fn time_to_collision(&self, other: &Container) -> Option<f64> {
        let dr = self.pos;
        let dv = self.vel;
        let dv_squared = dv.dot(&dv);

        let lhs = dv_squared * (self.r - other.r) * (self.r - other.r);
        let rhs = dr.cross_squared(&dv);

        if lhs < rhs {
            // equivalent to asking if discriminant < 0
            None
        } else {
            // find the smallest positive solution
            let disc = lhs - rhs;
            let r1 = -(dv.dot(&dr) + disc.sqrt()) / dv_squared;
            let r2 = -(dv.dot(&dr) - disc.sqrt()) / dv_squared;

            smallest_positive(r1, r2)
        }
    }

    fn collide(&mut self, other: &mut Container) -> Result<(), DynamicsError> {
        // Calculate and update the trajectory for a `Ball` colliding with a container, i.e.
        // a stationary `Ball` which we also assume totally contains `self`.

        let normed_normal = normalised_difference(self, other)?;
        let loc = normed_normal.anti_clockwise_perpendicular();

        let alpha = self.vel.dot(&loc);
        let beta = self.vel.dot(&normed_normal);

        self.set_vel(alpha * loc - beta * normed_normal);
        Ok(())
    }
}

fn smallest_positive(a: f64, b: f64) -> Option<f64> {
    let (x_min, x_max) = (a.min(b), a.max(b));
    if x_min.is_sign_positive() {
        Some(x_min)
    } else if x_max.is_sign_positive() {
        Some(x_max)
    } else {
        // If time to collision is 0, we also say that they don't collide
        // because the only way this happens in elastic dynamics is if they
        // were already stationary relative to each other.
        None
    }
}
