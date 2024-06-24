use crate::dynamics::ball::{Ball, Container};
use crate::dynamics::collide::Collide;
use crate::dynamics::maths::{approx_eq_f64, FloatVec};
use crate::dynamics::DynamicsError;
use itertools::Itertools;
// use pyo3::prelude::*;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Clone, Copy)]
enum CollisionPartner {
    Ball(usize),
    Container,
}

struct CollisionEvent {
    // Struct which identifies a collision between two `Ball`s within a `Simulation`.
    // The indices (`i`, `j`) of each `Ball` within the associated `Simulation`'s
    // `balls` `Vec` are stored, along with the time `t` at which the collision occurs.
    // A value for `j` of self.balls.len() indicates a collision with the container.
    // Finally, the velocities of the `Ball`s at the time the collision event is
    // registered is stored (`old_vels`) so that when the `CollisionEvent` is popped
    // from the `collisions` queue it can be verified that the `Ball`s have not
    // collided or changed velocity since.
    i: usize,
    j: CollisionPartner,
    t: f64,
    old_vels: (FloatVec, FloatVec),
}

impl PartialEq for CollisionEvent {
    fn eq(&self, other: &Self) -> bool {
        approx_eq_f64(self.t, other.t, 1)
    }
}

impl Eq for CollisionEvent {}

impl PartialOrd for CollisionEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CollisionEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.t.total_cmp(&other.t)
    }
}

impl Into<(usize, CollisionPartner, f64, (FloatVec, FloatVec))> for CollisionEvent {
    fn into(self) -> (usize, CollisionPartner, f64, (FloatVec, FloatVec)) {
        (self.i, self.j, self.t, self.old_vels)
    }
}

// #[pyclass(subclass)]
// #[pyo3(name = "_Simulation")]
pub struct Simulation {
    // #[pyo3(get)]
    global_time: f64,
    container: Container,
    balls: Vec<Ball>,
    collisions: BinaryHeap<Reverse<CollisionEvent>>,
}

impl Simulation {
    fn calculate_collision_event(&self, i: usize, j: usize) -> Option<CollisionEvent> {
        // Given two `Ball`s of the simulation at indices `i` and `j` of `balls`,
        // calculate the `CollisionEvent` between them, or return `None` if no
        // collision exists.
        let (p, q) = (&self.balls[i], &self.balls[j]);
        let time_to_collision_relative = p.time_to_collision(q)?;
        let t = self.global_time + time_to_collision_relative;
        let old_vels = (p.vel().to_owned(), q.vel().to_owned());
        let j = CollisionPartner::Ball(j);
        Some(CollisionEvent { i, j, t, old_vels })
    }

    fn calculate_container_collision(&self, i: usize) -> Option<CollisionEvent> {
        let ball = &self.balls[i];
        let container = &self.container;
        let time_to_collision_relative = ball.time_to_collision(container)?;
        let t = self.global_time + time_to_collision_relative;
        let old_vels = (ball.vel().to_owned(), FloatVec::origin());
        let j = CollisionPartner::Container;
        Some(CollisionEvent { i, j, t, old_vels })
    }

    fn generate_collision_queue(&mut self) {
        // Given a set of balls, calculate the order in which they will collide, assuming that
        // all their velocities remain constant. Store the collisions in a priority queue,
        // `self.collisions` so that the collisions can be efficiently looked up as the
        // `Simulation` runs.
        let n = self.balls.len();
        for pair in (0..n).combinations(2) {
            let (i, j) = (pair[0], pair[1]);
            if let Some(collision_event) = self.calculate_collision_event(i, j) {
                let queue = &mut self.collisions;
                queue.push(Reverse(collision_event));
            }
        }
    }

    fn generate_container_collisions(&mut self) {
        // Given a set of balls within a container, calculate the collisions of the balls
        // with the container, and push them in order to a collision queue.
        let n = self.balls.len();
        for i in 0..n {
            if let Some(collision_event) = self.calculate_container_collision(i) {
                let queue = &mut self.collisions;
                queue.push(Reverse(collision_event));
            }
        }
    }

    fn push_collisions(&mut self, i: usize) {
        // For a `Ball` at index `i` within the `self.balls` `Vec`, calculate
        // the collisions that will occur involving that `Ball`, and push them to the
        // collision queue.
        let n = self.balls.len();
        for j in 0..i {
            if let Some(collision_event) = self.calculate_collision_event(i, j) {
                self.collisions.push(Reverse(collision_event));
            }
        }

        for j in i + 1..n {
            if let Some(collision_event) = self.calculate_collision_event(i, j) {
                self.collisions.push(Reverse(collision_event));
            }
        }

        if let Some(collision_event) = self.calculate_container_collision(i) {
            self.collisions.push(Reverse(collision_event));
        }
    }

    pub fn step(&mut self, t: f64) {
        // Move the simulation forward in time by `t` seconds.
        for ball in self.balls.iter_mut() {
            ball.step(t)
        }
        self.global_time += t;
    }

    pub fn step_until(&mut self, t: f64) -> Result<(), DynamicsError> {
        // Move the simulation foward to time `t`. If `t` is behind the global time,
        // raise a corresponding error.
        let delta = t - self.global_time;
        if !delta.is_sign_positive() {
            // negative 0?
            Err(DynamicsError::SimulationFailure)
        } else {
            self.step(delta);
            Ok(())
        }
    }

    fn collide_by_index(&mut self, i: usize, j: usize) -> Result<(), DynamicsError> {
        // Calculate the new trajectories for `Ball`s at indices `i` and `j`
        // following a collision between them.

        // Because the `Ball`s are stored in a `Vec`, obtaining mutable
        // references to both of them simultaneously requires unsafe code. To
        // prevent multiple mutable references to the same memory from every
        // actually occuring, it is checked first that i and j are definitely
        // distinct.
        assert_ne!(i, j);
        let i_ptr = &mut self.balls[i] as *mut Ball;
        let j_ptr = &mut self.balls[j] as *mut Ball;

        unsafe {
            let ball_i = i_ptr.as_mut().unwrap();
            let ball_j = j_ptr.as_mut().unwrap();
            ball_i.collide(ball_j)?;
        }

        Ok(())
    }

    fn collide_members(&mut self, i: usize, j: CollisionPartner) -> Result<(), DynamicsError> {
        // Calculate the new trajectories for members of the `Simulation`, including
        // the `Ball`s as well as the `Container`. To indicate a collision with the
        // `Container`, `j` should be passed as the length of the `Vec` of `Ball`s.
        // If the `Container` is passed in through the first argument (i.e.
        // `i == self.balls.len())`, the simulation will panic.
        match j {
            CollisionPartner::Ball(j) => self.collide_by_index(i, j),
            CollisionPartner::Container => {
                let p = &mut self.balls[i];
                p.collide(&mut self.container)
            }
        }
    }

    fn next_collision(&mut self) -> Option<CollisionEvent> {
        // Pop the next collision from the queue. If it is still valid, i.e.
        // if the velocities of the involved `Ball`s have not changed, return
        // the `CollisionEvent`. Otherwise, return `None`.
        let reverse_collision = self.collisions.pop()?;
        let collision_info = reverse_collision.0;
        let (i, j, t, old_vels) = collision_info.into();
        let p = &self.balls[i];
        let q_vel = match j {
            CollisionPartner::Ball(j) => self.balls[j].vel,
            CollisionPartner::Container => FloatVec::origin(),
        }; // Just comparing 0f == 0f?
        let curr_vels = (p.vel, q_vel);
        if curr_vels == old_vels {
            Some(CollisionEvent { i, j, t, old_vels })
        } else {
            None
        }
    }

    fn next_collision_or_err(&mut self) -> Result<CollisionEvent, DynamicsError> {
        let mut collision_event = None;
        while collision_event.is_none() {
            collision_event = self.next_collision();
        }

        collision_event.ok_or(DynamicsError::SimulationFailure)
    }

    fn step_through_collision(&mut self) -> Result<(), DynamicsError> {
        // Run the simulation to and including the next collision that is scheduled
        // to occur. Calculate the dynamics of the collision and update the
        // collisions queue accordingly.
        let next_collision = self.next_collision_or_err()?;
        let (i, j, t, _) = next_collision.into();
        self.step_until(t)?;
        self.collide_members(i, j)?;
        self.push_collisions(i);
        if let CollisionPartner::Ball(j) = j {
            self.push_collisions(j);
        }
        Ok(())
    }

    pub fn run_collisions(&mut self, n: usize) -> Result<(), DynamicsError> {
        // Run the `Simulation` through `n` collisions.
        for _ in 0..n {
            self.step_through_collision()?;
        }

        Ok(())
    }
}
