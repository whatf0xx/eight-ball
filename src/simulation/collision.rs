use crate::dynamics::maths::{approx_eq_f64, FloatVec};

#[derive(Clone, Copy)]
pub enum CollisionPartner {
    Ball(usize),
    Container,
}

pub struct CollisionEvent {
    // Struct which identifies a collision between two `Ball`s within a
    // `Simulation`. `i` gives the index of the first ball involved in the
    // collision within the `Vec` of balls. `j` is an enum which can either
    // indicate a collision with the container, or with another ball in
    // the simulation, in which case the index is also stored. Finally, the
    // velocities of the `Ball`s at the time the collision event is registered
    // is stored (`old_vels`) so that when the `CollisionEvent` is popped from
    // the `collisions` queue it can be verified that the `Ball`s have not
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

impl From<CollisionEvent> for (usize, CollisionPartner, f64, (FloatVec, FloatVec)) {
    fn from(collision_event: CollisionEvent) -> Self {
        (
            collision_event.i,
            collision_event.j,
            collision_event.t,
            collision_event.old_vels,
        )
    }
}

impl CollisionEvent {
    pub fn new(i: usize, j: CollisionPartner, t: f64, old_vels: (FloatVec, FloatVec)) -> Self {
        CollisionEvent { i, j, t, old_vels }
    }
}
