use crate::Scalar;
#[must_use]
pub fn solve_linear(a: Scalar, b: Scalar) -> Vec<Scalar> {
    if a.abs() <= f64::EPSILON {
        Vec::new()
    } else {
        vec![-b/a]
    }
}
#[must_use]
pub fn solve_quadratic(a: Scalar, b: Scalar, c: Scalar) -> Vec<Scalar> {
    if a.abs() <= f64::EPSILON {
        return solve_linear(b, c)
    }
    let disc = b.mul_add(b, -4.0*a*c);
    if disc<0.0 {
        return Vec::new()
    }
    if disc.abs() <= f64::EPSILON {
        return vec![-b/(2.0*a)]
    }
    let s = disc.sqrt();
    let q = -0.5*(b+b.signum()*s);
    let mut roots = if q == 0.0 {
        vec![(-b+s)/(2.0*a), (-b-s)/(2.0*a)]
    } else {
        vec![q/a, c/q]
    };
    roots.sort_by(|x, y|x.total_cmp(y));
    roots.dedup_by(|x, y|(*x-*y).abs() <= 1e-13);
    roots
}
#[must_use]
pub fn solve_cubic(a: Scalar, b: Scalar, c: Scalar, d: Scalar) -> Vec<Scalar> {
    if a.abs() <= f64::EPSILON {
        return solve_quadratic(b, c, d)
    }
    let aa = b/a;
    let bb = c/a;
    let cc = d/a;
    let p = bb-aa*aa/3.0;
    let q = 2.0*aa.powi(3)/27.0-aa*bb/3.0+cc;
    let disc = (q*q)/4.0+(p*p*p)/27.0;
    let mut out = Vec::new();
    if disc>1e-15 {
        let s = disc.sqrt();
        let u = (-q/2.0+s).cbrt();
        let v = (-q/2.0-s).cbrt();
        out.push(u+v-aa/3.0)
    } else if disc.abs() <= 1e-15 {
        let u = (-q/2.0).cbrt();
        out.push(2.0*u-aa/3.0);
        out.push(-u-aa/3.0)
    } else {
        let r = 2.0*(-p/3.0).sqrt();
        let phi = ((3.0*q/(2.0*p))*((-3.0/p).sqrt())).acos()/3.0;
        for k in 0..3 {
            out.push(r*(phi-2.0*core::f64::consts::PI*(k as Scalar)/3.0).cos()-aa/3.0)
        }
    }
    out.sort_by(|x, y|x.total_cmp(y));
    out.dedup_by(|x, y|(*x-*y).abs() <= 1e-12);
    out
}
