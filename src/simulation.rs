use crate::dynamics::{Ball, DynamicsError};
use crate::maths::approx_eq_f64;
use itertools::Itertools;
use pyo3::prelude::*;
use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

struct CollisionEvent {
    // We add the field `v_dot_prod` so that we can keep track of whether or not this collision
    // is still relevant to the colliding balls; if either has experienced a collision since,
    // then its trajectory will be changed, and comparing the new value of `v_dot_prod` with the
    // stored one will return inequality.
    i: usize,
    j: usize,
    t: f64,
    v_dot_prod: f64,
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

fn generate_initial_collision_heap(balls: &[RefCell<Ball>]) -> BinaryHeap<Reverse<CollisionEvent>> {
    // Given a set of balls, calculate the order in which they will collide, assuming that
    // all their velocities remain constant.
    let mut heap = BinaryHeap::new();
    for enum_pair in balls.iter().enumerate().combinations(2) {
        let (i, p) = enum_pair[0];
        let (j, q) = enum_pair[1];
        let (p, q) = (p.borrow(), q.borrow());
        if let Some(t) = p.time_to_collision(&q) {
            let v_dot_prod = p.vel().dot(q.vel());
            heap.push(Reverse(CollisionEvent {
                i,
                j,
                t,
                v_dot_prod,
            }))
        }
    }

    heap
}

#[pyclass]
pub struct Simulation {
    global_time: f64,
    balls: Vec<RefCell<Ball>>,
    collisions: BinaryHeap<Reverse<CollisionEvent>>,
}

impl Simulation {
    pub fn new(balls: &mut [RefCell<Ball>]) -> Simulation {
        let balls = balls.to_vec();
        let collisions = generate_initial_collision_heap(&balls);
        Simulation {
            global_time: 0.0,
            balls: balls.to_vec(),
            collisions,
        }
    }

    pub fn step(&mut self, t: f64) {
        // Move the simulation forward in time by `t` seconds.
        for ball in self.balls.iter() {
            ball.borrow_mut().step(t)
        }
    }

    fn add_soonest_collision(&mut self, ball_index: usize) {
        // For the ball at the given index, calculate its next collision with
        // every other ball and push the resulting collision event to the event
        // queue. If no collision exists, do nothing.
        let ball = self.balls[ball_index].borrow();
        let (mut min_time, mut other_index) = (f64::INFINITY, None);

        let (left, right) = self.balls.split_at(ball_index);
        for (i, curr_other_ref) in left.iter().chain(right[1..].iter()).enumerate() {
            let curr_other = curr_other_ref.borrow();
            if let Some(time) = ball.time_to_collision(&curr_other) {
                if time < min_time {
                    min_time = time;
                    other_index = Some(i);
                }
            }
        }
        if let Some(j) = other_index {
            let v_dot_prod = ball.vel().dot(self.balls[j].borrow().vel());
            self.collisions.push(Reverse(CollisionEvent {
                i: ball_index,
                j,
                t: min_time,
                v_dot_prod,
            }));
        }
    }

    fn step_to_next_collision(&mut self) -> Result<(), DynamicsError> {
        // Step through to the next collision event that is scheduled to occur.
        // Check that the collision should indeed occur, and if so, execute it.
        // Calculate the next collision to occur for each of the collided balls,
        // and add them to the event queue.
        // If there are no collisions to execute, do nothing.
        if let Some(reverse_collision) = self.collisions.pop() {
            let col = reverse_collision.0; // this line just gets rid of the Reverse()
            let (op_p_refc, op_q_refc, v_dot_prod, t) = (
                self.balls.get(col.i),
                self.balls.get(col.j),
                col.v_dot_prod,
                col.t,
            );
            if let (Some(p_refc), Some(q_refc)) = (op_p_refc, op_q_refc) {
                // First, make sure that we actually point to valid members of the balls `Vec`.
                let calc_dot_prod = {
                    let (p, q) = (p_refc.borrow(), q_refc.borrow());
                    p.vel().dot(q.vel())
                };
                if approx_eq_f64(calc_dot_prod, v_dot_prod, 1) {
                    // This executes if the `CollisionEvent` is valid, i.e. the `Ball`s involved
                    // didn't change trajectories in the meantime.
                    self.step(t);
                    let (mut p, mut q) = (
                        self.balls[col.i].borrow_mut(),
                        self.balls[col.j].borrow_mut(),
                    );
                    p.collide(&mut q)?;
                    drop(p);
                    drop(q);

                    // Finally, work out the new CollisionEvents for each ball.
                    self.add_soonest_collision(col.i);
                    self.add_soonest_collision(col.j);
                }
            }
        }
        Ok(())
    }
}
