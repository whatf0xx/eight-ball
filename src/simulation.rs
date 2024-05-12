use crate::dynamics::Ball;
use crate::maths::SafeFloat;
use itertools::Itertools;
use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

struct CollisionEvent<'a> {
    // We add the field `v_dot_prod` so that we can keep track of whether or not this collision
    // is still relevant to the colliding balls; if either has experienced a collision since,
    // then its trajectory will be changed, and comparing the new value of `v_dot_prod` with the
    // stored one will return inequality.
    p: &'a RefCell<Ball>,
    q: &'a RefCell<Ball>,
    t: SafeFloat,
    v_dot_prod: SafeFloat,
}

impl CollisionEvent<'_> {
    pub fn new<'a>(p: &'a RefCell<Ball>, q: &'a RefCell<Ball>) -> Option<CollisionEvent<'a>> {
        let p_bor = p.borrow();
        let q_bor = q.borrow();
        let t = p_bor.time_to_collision(&q_bor)?;
        let v_dot_prod = p_bor.vel().dot(q_bor.vel());
        Some(CollisionEvent {
            p,
            q,
            t,
            v_dot_prod,
        })
    }
}

impl PartialEq for CollisionEvent<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.t.approx_eq(&other.t, 1)
    }
}

impl Eq for CollisionEvent<'_> {}

impl PartialOrd for CollisionEvent<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.t.partial_cmp(&other.t)
    }
}

impl Ord for CollisionEvent<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.t.cmp(&other.t)
    }
}

fn generate_initial_collision_heap<'a>(
    balls: &'a [RefCell<Ball>],
) -> BinaryHeap<Reverse<CollisionEvent<'a>>> {
    // Given a set of balls, calculate the order in which they will collide, assuming that
    // all their velocities remain constant.
    let mut heap = BinaryHeap::new();
    for pair in balls.iter().combinations(2) {
        let (p, q) = (pair[0], pair[1]);
        if let Some(collision) = CollisionEvent::new(&p, &q) {
            heap.push(Reverse(collision))
        }
    }

    heap
}
pub struct Simulation<'a> {
    global_time: SafeFloat,
    balls: Vec<RefCell<Ball>>,
    collisions: BinaryHeap<Reverse<CollisionEvent<'a>>>,
}

impl Simulation<'_> {
    pub fn new(balls: &mut [RefCell<Ball>]) -> Simulation {
        let collisions = generate_initial_collision_heap(balls);
        Simulation {
            global_time: SafeFloat::Zero,
            balls: balls.to_vec(),
            collisions,
        }
    }

    pub fn step(&mut self, t: SafeFloat) {
        // Move the simulation forward in time by `t` seconds.
        for ball in self.balls.iter() {
            ball.borrow_mut().step(t)
        }
    }

    fn step_to_next_collision(&mut self) {
        // Step through to the next collision event that is scheduled to occur.
        // Check that the collision should indeed occur, and if so, execute it.
        // If there are no collisions to execute, do nothing.
        if let Some(reverse_collision) = self.collisions.pop() {
            let col = reverse_collision.0; // this line just gets rid of the Reverse()
            let (p_refc, q_refc, v_dot_prod, t) = (col.p, col.q, col.v_dot_prod, col.t);
            let calc_dot_prod = {
                let (p, q) = (p_refc.borrow(), q_refc.borrow());
                p.vel().dot(q.vel())
            };
            if calc_dot_prod.approx_eq(&v_dot_prod, 1) {
                // This executes if the `CollisionEvent` is valid, i.e. the `Ball`s involved
                // didn't change trajectories in the meantime.
                self.step(t);
                let (mut p, mut q) = (p_refc.borrow_mut(), q_refc.borrow_mut());
                // Looking at the below, this might as well be wrapped up in a .collide() method
                // which can return Result<(), SafeFloatError> for the purpose of the currently
                // unused error handling. How do we do logs and warnings in Rust, anwyay?
                let (p_new_vel, q_new_vel) = p.calculate_collision(&q).unwrap();
                p.set_vel(p_new_vel);
                q.set_vel(q_new_vel);
            }
        }
    }
}
