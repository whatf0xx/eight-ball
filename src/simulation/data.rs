use super::event::CollisionPartner;
use crate::dynamics::maths::FloatVec;

/// The data we need for each ball in a data event entry.
pub struct BallData {
    pub sim_index: usize,
    pub position: FloatVec,
    pub old_vel: FloatVec,
    pub new_vel: FloatVec,
}

impl BallData {
    pub fn new(sim_index: usize, position: FloatVec, old_vel: FloatVec, new_vel: FloatVec) -> Self {
        BallData {
            sim_index,
            position,
            old_vel,
            new_vel,
        }
    }
}

/// The object that is collided with, sent in the data stream from the
/// simulation.
pub enum DataPartner {
    Ball(BallData),
    Container(f64),
}

enum PrePartner {
    Ball(usize, FloatVec, FloatVec),
    Container(f64),
}

/// Struct for holding all the data we could need directly before the collision
/// occurs. Combine with `PostData` to get the whole picture.
pub struct PreData {
    time: f64,
    sim_index: usize,
    position: FloatVec,
    old_vel: FloatVec,
    collision_partner: PrePartner,
}

pub struct PostData {
    new_vel: FloatVec,
    other_vel: Option<FloatVec>,
}

/// A chunk of data that completely describes a collision that occured within
/// the simulation
pub struct DataEvent {
    pub ball: BallData,
    pub other: DataPartner,
    pub time: f64,
}

impl From<(PreData, PostData)> for DataEvent {
    fn from(value: (PreData, PostData)) -> Self {
        let (pre_data, post_data) = value;
        let ball_data = BallData::new(
            pre_data.sim_index,
            pre_data.position,
            pre_data.old_vel,
            post_data.new_vel,
        );
        let other = match pre_data.collision_partner {
            PrePartner::Ball(j, pos, vel) => {
                // never get `None` variant, panic safe
                let new_vel = post_data.other_vel.unwrap();
                let ball = BallData::new(j, pos, vel, new_vel);
                DataPartner::Ball(ball)
            }
            PrePartner::Container(radius) => DataPartner::Container(radius),
        };
        DataEvent::new(ball_data, other, pre_data.time)
    }
}

impl DataEvent {
    pub fn new(ball: BallData, other: DataPartner, time: f64) -> Self {
        DataEvent { ball, other, time }
    }

    pub fn collision_centre(&self) -> FloatVec {
        let r_a = self.ball.position;
        match &self.other {
            DataPartner::Ball(ball_data) => {
                let r_b = ball_data.position;
                0.5 * (r_a + r_b)
            }
            DataPartner::Container(radius) => {
                let line = r_a.normalize();
                line * *radius
            }
        }
    }

    /// Calculate the momentum imparted on the container by the collision. If
    /// the collision is between two balls, then this will return the `None`
    /// variant, otherwise this will be equal to the magnitude in the change
    /// in momentum for the ball.
    pub fn container_pressure(&self) -> Option<f64> {
        match self.other {
            DataPartner::Ball(_) => None,
            DataPartner::Container(_) => {
                let delta_v = self.ball.old_vel - self.ball.new_vel;
                Some(delta_v.magnitude())
            }
        }
    }
}
