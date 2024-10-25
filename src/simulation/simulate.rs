use crate::dynamics::ball::{Ball, Container};
use crate::dynamics::collide::Collide;
use crate::dynamics::maths::FloatVec;
use crate::dynamics::DynamicsError;
use crate::simulation::event::{CollisionEvent, CollisionPartner, DataEvent};
use itertools::Itertools;
use pyo3::prelude::*;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

struct Params {
    delta: f64,
}

#[pyclass(subclass)]
#[pyo3(name = "_Simulation")]
pub struct Simulation {
    #[pyo3(get)]
    pub(crate) global_time: f64,
    params: Params,
    pub(crate) container: Container,
    pub(crate) balls: Vec<Ball>,
    pub(crate) collisions: BinaryHeap<Reverse<CollisionEvent>>,
}

impl Simulation {
    pub fn new(radius: f64) -> Simulation {
        let global_time = 0f64;
        let container = Container::new(radius);
        let balls = Vec::new();
        let collisions = BinaryHeap::new();
        let params = Params { delta: 1e-6 };
        Simulation {
            global_time,
            params,
            container,
            balls,
            collisions,
        }
    }

    fn calculate_collision_event(&self, i: usize, j: usize) -> Option<CollisionEvent> {
        // Given two `Ball`s of the simulation at indices `i` and `j` of `balls`,
        // calculate the `CollisionEvent` between them, or return `None` if no
        // collision exists.
        let (p, q) = (&self.balls[i], &self.balls[j]);
        let time_to_collision_relative = p.time_to_collision(q)?;
        let t = self.global_time + time_to_collision_relative;
        let old_vels = (p.vel().to_owned(), q.vel().to_owned());
        let j = CollisionPartner::Ball(j);
        Some(CollisionEvent::new(i, j, t, old_vels))
    }

    fn calculate_container_collision(&self, i: usize) -> Option<CollisionEvent> {
        let ball = &self.balls[i];
        let container = &self.container;
        let time_to_collision_relative = ball.time_to_collision(container)?;
        let t = self.global_time + time_to_collision_relative;
        let old_vels = (ball.vel().to_owned(), FloatVec::origin());
        let j = CollisionPartner::Container;
        Some(CollisionEvent::new(i, j, t, old_vels))
    }

    pub(crate) fn generate_collision_queue(&mut self) {
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

    pub(crate) fn generate_container_collisions(&mut self) {
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
            ball.step(t * (1. - self.params.delta))
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
            Some(CollisionEvent::new(i, j, t, old_vels))
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

    pub(crate) fn step_through_collision(&mut self) -> Result<(), DynamicsError> {
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

    /// Run the simulation through the next collision, as above, but publish
    /// the data associated with the collision as a `DataEvent` that can be
    /// streamed.
    fn step_with_data(&mut self) -> Result<DataEvent, DynamicsError> {
        let next_collision = self.next_collision_or_err()?;
        let (i, j, t, _) = next_collision.into();
        self.step_until(t)?;
        // This is when the collision happens
        let time = self.global_time;
        let old_vels_a = self.balls[i].vel;
        // match j {
        //     CollisionPartner::Ball(j) => {
        //         let old_vels_b =
        //     }
        // }
        todo!();
    }

    pub fn run_collisions(&mut self, n: usize) -> Result<(), DynamicsError> {
        // Run the `Simulation` through `n` collisions.
        for _ in 0..n {
            self.step_through_collision()?;
        }

        Ok(())
    }
}
