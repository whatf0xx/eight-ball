use crate::dynamics::dynamics::{Ball, DynamicsError};
use crate::dynamics::maths::approx_eq_f64;
use itertools::Itertools;
use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;
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

#[pyclass(subclass)]
#[pyo3(name = "_Simulation")]
pub struct Simulation {
    #[pyo3(get)]
    global_time: f64,
    balls: Vec<Ball>,
    collisions: BinaryHeap<Reverse<CollisionEvent>>,
}

// #[pymethods]
// impl Simulation {
//     #[new]
//     pub fn py_new() -> Simulation {
//         Simulation {
//             global_time: 0.0,
//             balls: Vec::new(),
//             collisions: BinaryHeap::new(),
//         }
//     }

//     pub fn add_ball(&mut self, ball: Py<Ball>) {
//         self.balls.push(ball)
//     }

//     #[getter]
//     pub fn get_balls(&self) -> Vec<Py<Ball>> {
//         self.balls
//             .iter()
//             .map(|ball| Python::with_gil(|py| Py::clone_ref(ball, py)))
//             .collect()
//     }

//     pub fn ball_by_index(&self, index: usize) -> PyResult<Py<Ball>> {
//         match self.balls.get(index) {
//             Some(ball) => Ok(Python::with_gil(|py| Py::clone_ref(ball, py))),
//             None => Err(PyIndexError::new_err(format!(
//                 "self.balls has length {} but index given of {}",
//                 self.balls.len(),
//                 index
//             ))),
//         }
//     }
// }

impl Simulation {
    // pub fn new(balls: &[Ball]) -> Simulation {
    //     let balls = balls.iter().map(|ball| Py::new())
    //     let mut sim = Simulation {
    //         global_time: 0.0,
    //         balls: balls,
    //         collisions: BinaryHeap::new(),
    //     };
    //     sim.generate_collision_heap();
    //     sim
    // }

    fn generate_collision_heap(&mut self) {
        // Given a set of balls, calculate the order in which they will collide, assuming that
        // all their velocities remain constant.
        let heap = &mut self.collisions;
        for enum_pair in self.balls.iter().enumerate().combinations(2) {
            let (i, p) = enum_pair[0];
            let (j, q) = enum_pair[1];
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
    }

    pub fn step(&mut self, t: f64) {
        // Move the simulation forward in time by `t` seconds.
        for ball in self.balls.iter_mut() {
            ball.step(t)
        }
    }

    fn add_soonest_collision(&mut self, ball_index: usize) {
        // For the ball at the given index, calculate its next collision with
        // every other ball and push the resulting collision event to the event
        // queue. If no collision exists, do nothing.
        let ball = &self.balls[ball_index];
        let (mut min_time, mut other_index) = (f64::INFINITY, None);

        let (left, right) = self.balls.split_at(ball_index);
        let (is, js) = (0..ball_index, ball_index + 1..self.balls.len());
        let balls_enum = is.zip(left.iter()).chain(js.zip(right.iter()));

        for (i, curr_other_ref) in balls_enum {
            let curr_other = curr_other_ref;
            if let Some(time) = ball.time_to_collision(&curr_other) {
                if time < min_time {
                    min_time = time;
                    other_index = Some(i);
                }
            }
        }

        if let Some(j) = other_index {
            let v_dot_prod = ball.vel().dot(self.balls[j].vel());
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
            let (p, q, v_dot_prod, t) = (
                &self.balls[col.i],
                &self.balls[col.j],
                col.v_dot_prod,
                col.t,
            );
            let calc_dot_prod = p.vel().dot(q.vel());
            if approx_eq_f64(calc_dot_prod, v_dot_prod, 1) {
                // This executes if the `CollisionEvent` is valid, i.e. the `Ball`s involved
                // didn't change trajectories in the meantime.
                self.step(t);
                let mut p_local = self.balls[col.i].clone();
                let q = &mut self.balls[col.j];
                // let (p, q) = (&mut self.balls[col.i], &mut self.balls[col.j]);
                p_local.collide(q)?;

                self.balls[col.i] = p_local;

                // Finally, work out the new CollisionEvents for each ball.
                self.add_soonest_collision(col.i);
                self.add_soonest_collision(col.j);
            }
        }
        Ok(())
    }
}
