use crate::dynamics::ball::Ball;
use crate::dynamics::collide::Collide;
use crate::dynamics::maths::{approx_eq_f64, FloatVec};
use crate::dynamics::DynamicsError;
use itertools::Itertools;
// use pyo3::prelude::*;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

struct CollisionEvent {
    // Struct which identifies a collision between two `Ball`s within a `Simulation`.
    // The indices (`i`, `j`) of each `Ball` within the associated `Simulation`'s
    // `balls` `Vec` are stored, along with the time `t` at which the collision occurs.
    // Finally, the velocities of the `Ball`s at the time the collision event is
    // registered is stored (`old_vels`) so that when the `CollisionEvent` is popped
    // from the `collisions` queue it can be verified that the `Ball`s have not
    // collided or changed velocity since.
    i: usize,
    j: usize,
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

// #[pyclass(subclass)]
// #[pyo3(name = "_Simulation")]
pub struct Simulation {
    // #[pyo3(get)]
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
    pub fn new(balls: &[Ball]) -> Simulation {
        // Given an immutable slice of `Ball`s, `balls`, generate a simulation
        // of their interactions. This assumes that the first ball in the slice
        // is the container.
        let balls = balls.to_vec();
        let mut sim = Simulation {
            global_time: 0.0,
            balls,
            collisions: BinaryHeap::new(),
        };
        sim.generate_collision_queue();
        sim
    }

    fn calculate_collision_event(&self, i: usize, j: usize) -> Option<CollisionEvent> {
        // Given two `Ball`s of the simulation at indices `i` and `j` of `balls`,
        // calculate the `CollisionEvent` between them, or return `None` if no
        // collision exists.
        let (p, q) = (&self.balls[i], &self.balls[j]);
        let time_to_collision_relative = p.time_to_collision(q)?;
        let t = self.global_time + time_to_collision_relative;
        let old_vels = (p.vel().to_owned(), q.vel().to_owned());
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

    fn push_collisions(&mut self, i: usize) {
        todo!("Check the Python code for how to neatly handle collisions with the container")
    }

    pub fn step(&mut self, t: f64) {
        // Move the simulation forward in time by `t` seconds.
        for ball in self.balls.iter_mut() {
            ball.step(t)
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

    // fn step_to_next_collision(&mut self) -> Result<(), DynamicsError> {
    //     // Step through to the next collision event that is scheduled to occur.
    //     // Check that the collision should indeed occur, and if so, execute it.
    //     // Calculate the next collision to occur for each of the collided balls,
    //     // and add them to the event queue.
    //     // If there are no collisions to execute, do nothing.
    //     if let Some(reverse_collision) = self.collisions.pop() {
    //         let col = reverse_collision.0; // this line just gets rid of the Reverse()
    //         let (p, q, v_dot_prod, t) =
    //             (&self.balls[col.i], &self.balls[col.j], col.old_vels, col.t);
    //         let calc_dot_prod = p.vel().dot(q.vel());
    //         if approx_eq_f64(calc_dot_prod, v_dot_prod, 1) {
    //             // This executes if the `CollisionEvent` is valid, i.e. the `Ball`s involved
    //             // didn't change trajectories in the meantime.
    //             self.step(t);
    //             self.collide_by_index(col.i, col.j)?;

    //             // Finally, work out the new CollisionEvents for each ball.
    //             self.add_soonest_collision(col.i);
    //             self.add_soonest_collision(col.j);
    //         }
    //     }
    //     Ok(())
    // }
}
