use core::ops::{Add, Mul};
use num_traits::float::FloatConst;
use num_traits::real::Real;
use smallvec::SmallVec;

#[inline]
pub fn min<T>(a: T, b: T) -> T
where
    T: PartialOrd<T>,
{
    if b < a {
        b
    } else {
        a
    }
}

#[inline]
pub fn max<T>(a: T, b: T) -> T
where
    T: PartialOrd<T>,
{
    if b < a {
        b
    } else {
        a
    }
}

#[inline]
pub fn median<T>(a: T, b: T, c: T) -> T
where
    T: PartialOrd<T> + Copy,
{
    max(min(a, b), min(max(a, b), c))
}

#[inline]
pub fn mix<T, S: num_traits::Num + Copy>(a: T, b: T, weight: S) -> T
where
    T: Mul<S, Output = T> + Add<T, Output = T>,
{
    a * (S::one() - weight) + b * weight
}

/// Solves the linear equation `b*x + a = 0`.
pub fn solve_linear<S: Real>(b: S, a: S) -> Option<S> {
    if b.abs() > S::epsilon() {
        Some(-a / b)
    } else if a.abs() <= S::epsilon() {
        Some(S::zero())
    } else {
        None
    }
}

/// Solves the quadratic equation `c*x^2 + b*x + a = 0`.
pub fn solve_quadratic<S: Real>(c: S, b: S, a: S) -> SmallVec<[S; 2]> {
    let mut solution = SmallVec::new();
    if c.abs() < S::epsilon() {
        if let Some(value) = solve_linear(b, a) {
            solution.push(value);
        }
    } else {
        let _2 = S::one() + S::one();
        let _4 = _2 + _2;
        let dscr = b * b - _4 * c * a;
        let two_c = _2 * c;
        if dscr > S::epsilon() {
            let dscr = dscr.sqrt();
            solution.push((-b + dscr) / two_c);
            solution.push((-b - dscr) / two_c);
        } else if !(dscr < -S::epsilon()) {
            solution.push(-b / two_c);
        }
    }
    solution
}

/// Solves the depressed cubic equation `x^3 + b*x + a = 0`.
pub fn solve_cubic_depressed<S: Real + FloatConst>(b: S, a: S) -> SmallVec<[S; 3]> {
    let mut solution = SmallVec::new();
    if b.abs() < S::epsilon() {
        solution.push(-a.cbrt());
    } else if a.abs() < S::epsilon() {
        solution.append(&mut solve_quadratic(S::one(), S::zero(), b));
        solution.push(S::zero());
    } else {
        let _2 = S::one() + S::one();
        let _3 = _2 + S::one();
        let _4 = _2 + _2;
        let _9 = _3 + _3 + _3;
        let _27 = _9 + _9 + _9;

        let d = a * a / _4 + b * b * b / _27;
        if d < S::zero() {
            let sq = (-_4 * b / _3).sqrt();
            let phi = (-_4 * a / (sq * sq * sq)).acos() / _3;
            let two_third_pi = _2 * S::FRAC_PI_3();
            solution.push(sq * phi.cos());
            solution.push(sq * (phi + two_third_pi).cos());
            solution.push(sq * (phi - two_third_pi).cos());
        } else {
            let sq = d.sqrt();
            let a_half = a / _2;
            let x1 = (sq - a_half).cbrt() - (sq + a_half).cbrt();
            solution.push(x1);
            if d.abs() < S::epsilon() {
                solution.push(a_half);
            }
        }
    }
    solution
}

/// Solves the normalized cubic equation `x^3 + c*x^2 + b*x + a = 0`.
pub fn solve_cubic_normalized<S: Real + FloatConst>(c: S, b: S, a: S) -> SmallVec<[S; 3]> {
    if c.abs() < S::epsilon() {
        solve_cubic_depressed(b, c)
    } else {
        let mut solution = SmallVec::new();

        let _2 = S::one() + S::one();
        let _3 = _2 + S::one();
        let _4 = _2 + _2;
        let _6 = _3 + _3;
        let _9 = _3 + _3 + _3;
        let _27 = _9 + _9 + _9;
        let _54 = _27 + _27;

        let c_squared = c * c;
        let q = (_3 * b - c_squared) / _9;
        let r = (_9 * c * b - _27 * a - _2 * c_squared * c) / _54;
        let q3 = q * q * q;
        let d = q3 + r * r;
        let c_thirds = c / _3;

        if d < -S::epsilon() {
            let phi_3 = (r / (-q3).sqrt()).acos() / _3;
            let sqrt_q_2 = _2 * (-q).sqrt();
            let two_third_pi = _2 * S::FRAC_PI_3();
            solution.push(sqrt_q_2 * phi_3.cos() - c_thirds);
            solution.push(sqrt_q_2 * (phi_3 - two_third_pi).cos() - c_thirds);
            solution.push(sqrt_q_2 * (phi_3 + two_third_pi).cos() - c_thirds);
        } else {
            let d = d.sqrt();
            let s = (r + d).cbrt();
            let t = (r - d).cbrt();

            solution.push(s + t - c_thirds);
            if (s - t).abs() < S::epsilon() && (s + t) > S::epsilon() {
                solution.push(-(s + t) / _2 - c_thirds);
            }
        }
        solution
    }
}

/// Solves the cubic equation `d*x^3 + c*x^2 + b*x + a = 0`.
pub fn solve_cubic<S: Real + FloatConst>(d: S, c: S, b: S, a: S) -> SmallVec<[S; 3]> {
    if d.abs() < S::epsilon() {
        let mut solution = SmallVec::new();
        solution.append(&mut solve_quadratic(c, b, a));
        solution
    } else if c.abs() < S::epsilon() {
        solve_cubic_depressed(b / d, a / d)
    } else {
        solve_cubic_normalized(b / d, c / d, a / d)
    }
}
