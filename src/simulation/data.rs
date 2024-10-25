use crate::dynamics::{ball::Ball, maths::FloatVec};

use super::{event::CollisionPartner, simulate::Simulation};

/// A chunk of data that represents the state of the collision directly before
/// it occurs
pub enum PreData {
    BallCollision {
        time: f64,
        indices: (usize, usize),
        pres: (Ball, Ball),
    },
    ContainerCollision {
        time: f64,
        index: usize,
        pre: Ball,
    },
}

impl PreData {
    pub fn from_indices(sim: &Simulation, i: usize, j: CollisionPartner) -> Self {
        let time = sim.global_time;
        let ball = sim.balls[i].clone();
        match j {
            CollisionPartner::Ball(j) => {
                let other = sim.balls[j].clone();
                let indices = (i, j);
                let pres = (ball, other);
                PreData::BallCollision {
                    time,
                    indices,
                    pres,
                }
            }
            CollisionPartner::Container => {
                let index = i;
                let pre = ball;
                PreData::ContainerCollision { time, index, pre }
            }
        }
    }
}

pub enum PostData {
    BallCollision { posts: (Ball, Ball) },
    ContainerCollision { post: Ball },
}

impl PostData {
    pub fn from_indices(sim: &Simulation, i: usize, j: CollisionPartner) -> Self {
        let ball = sim.balls[i].clone();
        match j {
            CollisionPartner::Ball(j) => {
                let other = sim.balls[j].clone();
                let posts = (ball, other);
                PostData::BallCollision { posts }
            }
            CollisionPartner::Container => PostData::ContainerCollision { post: ball },
        }
    }
}

/// A chunk of data that completely describes a collision that occured within
/// the simulation
pub enum DataEvent {
    BallCollision {
        time: f64,
        indices: (usize, usize),
        pres: (Ball, Ball),
        posts: (Ball, Ball),
    },
    ContainerCollision {
        time: f64,
        index: usize,
        pre: Ball,
        post: Ball,
    },
}

impl From<(PreData, PostData)> for DataEvent {
    fn from(value: (PreData, PostData)) -> Self {
        match value {
            (
                PreData::BallCollision {
                    time,
                    indices,
                    pres,
                },
                PostData::BallCollision { posts },
            ) => DataEvent::BallCollision {
                time,
                indices,
                pres,
                posts,
            },
            (
                PreData::ContainerCollision { time, index, pre },
                PostData::ContainerCollision { post },
            ) => DataEvent::ContainerCollision {
                time,
                index,
                pre,
                post,
            },
            (
                PreData::BallCollision {
                    time,
                    indices,
                    pres,
                },
                PostData::ContainerCollision { post },
            ) => panic!(),
            (
                PreData::ContainerCollision { time, index, pre },
                PostData::BallCollision { posts },
            ) => panic!(),
        }
    }
}

// impl DataEvent {
//     pub fn new(ball: BallData, other: DataPartner, time: f64) -> Self {
//         DataEvent { ball, other, time }
//     }

//     pub fn collision_centre(&self) -> FloatVec {
//         let r_a = self.ball.position;
//         match &self.other {
//             DataPartner::Ball(ball_data) => {
//                 let r_b = ball_data.position;
//                 0.5 * (r_a + r_b)
//             }
//             DataPartner::Container(radius) => {
//                 let line = r_a.normalize();
//                 line * *radius
//             }
//         }
//     }

//     /// Calculate the momentum imparted on the container by the collision. If
//     /// the collision is between two balls, then this will return the `None`
//     /// variant, otherwise this will be equal to the magnitude in the change
//     /// in momentum for the ball.
//     pub fn container_pressure(&self) -> Option<f64> {
//         match self.other {
//             DataPartner::Ball(_) => None,
//             DataPartner::Container(_) => {
//                 let delta_v = self.ball.old_vel - self.ball.new_vel;
//                 Some(delta_v.magnitude())
//             }
//         }
//     }
// }
