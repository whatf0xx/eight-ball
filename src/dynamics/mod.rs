use pyo3::prelude::*;
pub mod ball;
pub mod maths;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn time_to_collision() {
        let b1 = ball::Ball::new((0., 0.).into(), (1., 0.).into(), 0.1);
        let b2 = ball::Ball::new((1., 0.).into(), (0., 0.).into(), 0.1);
        let ttc = b1.time_to_collision(&b2).unwrap();
        println!("{}", ttc);

        assert!(maths::approx_eq_f64(ttc, 0.8, 1));
    }
}
