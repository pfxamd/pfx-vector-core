use crate::{CoreError, CoreResult, Scalar, Tolerance};
fn simpson<F: Fn(Scalar) -> Scalar>(f: &F, a: Scalar, b: Scalar) -> Scalar {
    let c = (a + b) * 0.5;
    (b - a) / 6.0 * (f(a) + 4.0 * f(c) + f(b))
}
fn recurse<F: Fn(Scalar) -> Scalar>(
    f: &F,
    a: Scalar,
    b: Scalar,
    eps: Scalar,
    whole: Scalar,
    depth: u32,
) -> CoreResult<Scalar> {
    let c = (a + b) * 0.5;
    let left = simpson(f, a, c);
    let right = simpson(f, c, b);
    let delta = left + right - whole;
    if depth == 0 {
        if delta.abs() <= 15.0 * eps {
            return Ok(left + right + delta / 15.0);
        }
        return Err(CoreError::IterationLimit);
    }
    if delta.abs() <= 15.0 * eps {
        Ok(left + right + delta / 15.0)
    } else {
        Ok(recurse(f, a, c, eps * 0.5, left, depth - 1)?
            + recurse(f, c, b, eps * 0.5, right, depth - 1)?)
    }
}
pub fn adaptive_simpson<F: Fn(Scalar) -> Scalar>(
    f: F,
    a: Scalar,
    b: Scalar,
    tol: Tolerance,
) -> CoreResult<Scalar> {
    if a == b {
        return Ok(0.0);
    }
    let whole = simpson(&f, a, b);
    let eps = tol.absolute.max(tol.relative * whole.abs()).max(1e-12);
    recurse(&f, a, b, eps, whole, 24)
}
