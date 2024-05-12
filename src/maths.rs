use std::{cmp, ops};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum SafeFloat {
    SafeNonZero(f64),
    Zero,
}

#[derive(Debug)]
pub enum SafeFloatError {
    FromNanError,
    FromInfinityError,
    FromZeroError,
    DivideByZeroError,
    BadVecError,
}

impl TryFrom<f64> for SafeFloat {
    type Error = SafeFloatError;
    fn try_from(f: f64) -> Result<SafeFloat, SafeFloatError> {
        if f.is_nan() {
            Err(SafeFloatError::FromNanError)
        } else if f.is_infinite() {
            Err(SafeFloatError::FromInfinityError)
        } else if f == 0.0 {
            Err(SafeFloatError::FromZeroError)
        } else {
            Ok(SafeFloat::SafeNonZero(f))
        }
    }
}
use SafeFloat::{SafeNonZero, Zero};

impl SafeFloat {
    pub fn zero() -> SafeFloat {
        // Generate a new instance of a safe zero.
        Zero
    }
}

impl From<SafeFloat> for f64 {
    fn from(sf: SafeFloat) -> f64 {
        match sf {
            SafeNonZero(f) => f,
            Zero => 0.0,
        }
    }
}

impl ops::Add<SafeFloat> for SafeFloat {
    type Output = SafeFloat;

    fn add(self, other: SafeFloat) -> Self::Output {
        match (self, other) {
            (Zero, Zero) => Zero,
            (Zero, SafeNonZero(f)) => SafeNonZero(f),
            (SafeNonZero(f), Zero) => SafeNonZero(f),
            (SafeNonZero(f), SafeNonZero(g)) => SafeNonZero(f + g),
        }
    }
}

impl ops::Sub<SafeFloat> for SafeFloat {
    type Output = SafeFloat;

    fn sub(self, other: SafeFloat) -> Self::Output {
        match (self, other) {
            (Zero, Zero) => Zero,
            (Zero, SafeNonZero(f)) => SafeNonZero(f),
            (SafeNonZero(f), Zero) => SafeNonZero(f),
            (SafeNonZero(f), SafeNonZero(g)) => SafeNonZero(f - g),
        }
    }
}

impl ops::Neg for SafeFloat {
    type Output = SafeFloat;

    fn neg(self) -> Self::Output {
        match self {
            Zero => Zero,
            SafeNonZero(f) => SafeNonZero(-f),
        }
    }
}

impl ops::Mul<SafeFloat> for SafeFloat {
    type Output = SafeFloat;

    fn mul(self, other: SafeFloat) -> Self::Output {
        match (self, other) {
            (Zero, Zero) => Zero,
            (Zero, SafeNonZero(_)) => Zero,
            (SafeNonZero(_), Zero) => Zero,
            (SafeNonZero(f), SafeNonZero(g)) => SafeNonZero(f * g),
        }
    }
}

impl ops::Div<SafeFloat> for SafeFloat {
    type Output = Result<SafeFloat, SafeFloatError>;

    fn div(self, other: SafeFloat) -> Self::Output {
        match (self, other) {
            (Zero, Zero) => Err(SafeFloatError::DivideByZeroError),
            (Zero, SafeNonZero(_)) => Ok(Zero),
            (SafeNonZero(_), Zero) => Err(SafeFloatError::DivideByZeroError),
            (SafeNonZero(f), SafeNonZero(g)) => Ok(SafeNonZero(f / g)),
        }
    }
}

impl SafeFloat {
    pub fn sqrt(&self) -> SafeFloat {
        match self {
            SafeNonZero(f) => SafeNonZero(f.sqrt()),
            Zero => Zero,
        }
    }
}

impl SafeFloat {
    pub fn is_positive(&self) -> bool {
        match self {
            Zero => false,
            SafeNonZero(f) => {
                if *f > 0.0 {
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn is_negative(&self) -> bool {
        match self {
            Zero => false,
            SafeNonZero(f) => {
                if *f < 0.0 {
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Zero => true,
            SafeNonZero(_) => false,
        }
    }
}

impl Eq for SafeFloat {} // We can say this because we exclude NaN, which is not Eq

fn approx_eq_f64(a: f64, b: f64, ulp: u64) -> bool {
    let au = a.to_bits();
    let bu = b.to_bits();

    let diff = au.max(bu) - au.min(bu);
    if diff <= ulp {
        true
    } else {
        false
    }
}

impl SafeFloat {
    pub fn approx_eq(&self, other: &SafeFloat, ulp: u64) -> bool {
        match (self, other) {
            (Zero, Zero) => true,
            (Zero, SafeNonZero(_)) => false,
            (SafeNonZero(_), Zero) => false,
            (SafeNonZero(f), SafeNonZero(g)) => approx_eq_f64(*f, *g, ulp),
        }
    }
}

impl Ord for SafeFloat {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match (self, other) {
            (Zero, Zero) => cmp::Ordering::Equal,
            (Zero, SafeNonZero(f)) => {
                if f.is_sign_negative() {
                    cmp::Ordering::Greater
                } else {
                    cmp::Ordering::Less
                }
            }
            (SafeNonZero(f), Zero) => {
                if f.is_sign_positive() {
                    cmp::Ordering::Greater
                } else {
                    cmp::Ordering::Less
                }
            }
            (SafeNonZero(f), SafeNonZero(g)) => {
                if approx_eq_f64(*f, *g, 1) {
                    cmp::Ordering::Equal
                } else if f > g {
                    cmp::Ordering::Greater
                } else {
                    cmp::Ordering::Less
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct SafeFloatVec {
    x: SafeFloat,
    y: SafeFloat,
}

impl ops::Add<SafeFloatVec> for SafeFloatVec {
    type Output = SafeFloatVec;

    fn add(self, other: SafeFloatVec) -> Self::Output {
        SafeFloatVec {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::Sub<SafeFloatVec> for SafeFloatVec {
    type Output = SafeFloatVec;

    fn sub(self, other: SafeFloatVec) -> Self::Output {
        SafeFloatVec {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl ops::Mul<SafeFloat> for SafeFloatVec {
    type Output = SafeFloatVec;

    fn mul(self, other: SafeFloat) -> Self::Output {
        SafeFloatVec {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl ops::Mul<SafeFloatVec> for SafeFloat {
    type Output = SafeFloatVec;

    fn mul(self, other: SafeFloatVec) -> Self::Output {
        other * self
    }
}

impl ops::Div<SafeFloat> for SafeFloatVec {
    type Output = Result<SafeFloatVec, SafeFloatError>;

    fn div(self, rhs: SafeFloat) -> Self::Output {
        match rhs {
            SafeNonZero(_) => Ok(SafeFloatVec {
                x: (self.x / rhs).unwrap(),
                y: (self.y / rhs).unwrap(),
            }),
            Zero => Err(SafeFloatError::DivideByZeroError),
        }
    }
}

impl SafeFloatVec {
    pub fn new(x: SafeFloat, y: SafeFloat) -> SafeFloatVec {
        SafeFloatVec { x, y }
    }

    pub fn from_floats(x: f64, y: f64) -> Result<SafeFloatVec, SafeFloatError> {
        let x = SafeFloat::try_from(x)?;
        let y = SafeFloat::try_from(y)?;
        Ok(SafeFloatVec { x, y })
    }

    pub fn dot(&self, other: &SafeFloatVec) -> SafeFloat {
        self.x * other.x + self.y * other.y
    }

    pub fn cross_squared(&self, other: &SafeFloatVec) -> SafeFloat {
        let x = self.x * other.y - self.y * other.x;
        x * x
    }

    pub fn anti_clockwise_perpendicular(&self) -> SafeFloatVec {
        SafeFloatVec {
            x: -self.y,
            y: self.x,
        }
    }

    pub fn approx_eq(&self, other: &SafeFloatVec, ulp: u64) -> bool {
        // Determine if the two SafeFloatVecs are equal to within
        // `n` units in the last place. As `n` is by default 1 and
        // otherwise assumed to be very small, this determines if
        // the SafeFloatVecs are as close together as floating-
        // point arithmetic allows.
        if self.x.approx_eq(&other.x, ulp) && self.y.approx_eq(&other.y, ulp) {
            true
        } else {
            false
        }
    }
}
