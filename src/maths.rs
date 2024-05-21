use std::ops;

pub fn approx_eq_f64(a: f64, b: f64, ulp: u64) -> bool {
    let au = a.to_bits();
    let bu = b.to_bits();

    let diff = au.max(bu) - au.min(bu);
    diff <= ulp
}

#[derive(Clone, Copy, Default)]
pub struct FloatVec {
    x: f64,
    y: f64,
}

impl ops::Add<FloatVec> for FloatVec {
    type Output = FloatVec;

    fn add(self, other: FloatVec) -> Self::Output {
        FloatVec {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::Sub<FloatVec> for FloatVec {
    type Output = FloatVec;

    fn sub(self, other: FloatVec) -> Self::Output {
        FloatVec {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl ops::Mul<f64> for FloatVec {
    type Output = FloatVec;

    fn mul(self, other: f64) -> Self::Output {
        FloatVec {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl ops::Mul<FloatVec> for f64 {
    type Output = FloatVec;

    fn mul(self, other: FloatVec) -> Self::Output {
        other * self
    }
}

impl ops::Div<f64> for FloatVec {
    type Output = FloatVec;

    fn div(self, rhs: f64) -> Self::Output {
        FloatVec {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl FloatVec {
    pub fn new(x: f64, y: f64) -> FloatVec {
        FloatVec { x, y }
    }

    pub fn dot(&self, other: &FloatVec) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn cross_squared(&self, other: &FloatVec) -> f64 {
        let x = self.x * other.y - self.y * other.x;
        x * x
    }

    pub fn anti_clockwise_perpendicular(&self) -> FloatVec {
        FloatVec {
            x: -self.y,
            y: self.x,
        }
    }

    pub fn approx_eq(&self, other: &FloatVec, ulp: u64) -> bool {
        // Determine if the two SafeFloatVecs are equal to within
        // `n` units in the last place. As `n` is by default 1 and
        // otherwise assumed to be very small, this determines if
        // the SafeFloatVecs are as close together as floating-
        // point arithmetic allows.
        approx_eq_f64(self.x, other.x, ulp) && approx_eq_f64(self.y, other.y, ulp)
    }
}
