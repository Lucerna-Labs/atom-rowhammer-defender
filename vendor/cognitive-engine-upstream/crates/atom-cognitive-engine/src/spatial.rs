//! BUILD-9 — spatial reasoning: orientation, frames, directions.
//! Precommitted in `docs/BUILD9-SPATIAL.md` BEFORE this code, under
//! vault Rules 1/2/3 and project rules 0/0a/0b.
//!
//! The layer BUILD-8 lacked: SSS congruence is blind to handedness;
//! this module holds the orientation and frame structure. Rotations are
//! unitaries (exact 90° on the grid); reflection is conjugation;
//! directions are the lattice's own sign classes.
//!
//! AUDIT: N, the lattice axes/sign classes (structural), the rotation
//! operators (unitaries). Queries carry no scene content.

use crate::geometry::{min_image, GeometryField, Point};

/// Rotate a point by 90° counter-clockwise: (x, y) -> (-y, x) — a
/// unitary operation on the grid.
pub fn rotate90(p: Point) -> Point {
    min_image((-p.1, p.0))
}

/// Rotate by 180°.
pub fn rotate180(p: Point) -> Point {
    min_image((-p.0, -p.1))
}

/// Rotate by 270°.
pub fn rotate270(p: Point) -> Point {
    min_image((p.1, -p.0))
}

/// Reflect across the y-axis: conjugation — the handedness flip.
pub fn reflect(p: Point) -> Point {
    min_image((-p.0, p.1))
}

/// Signed area (twice) of the triangle from min-image edges — the
/// chirality carrier. Sign is handedness; the metric (side lengths)
/// cannot see it.
pub fn signed_area(triangle: &[Point; 3]) -> i64 {
    let (a, b, c) = (triangle[0], triangle[1], triangle[2]);
    let e1 = min_image((b.0 - a.0, b.1 - a.1));
    let e2 = min_image((c.0 - a.0, c.1 - a.1));
    e1.0 * e2.1 - e1.1 * e2.0
}

/// Handedness: +1 counter-clockwise, -1 clockwise, 0 degenerate.
pub fn handedness(triangle: &[Point; 3]) -> i8 {
    match signed_area(triangle).signum() {
        1 => 1,
        -1 => -1,
        _ => 0,
    }
}

/// FRAME EQUIVARIANCE: the formed field of a rotated scene equals the
/// rotated field of the scene (the spatial identity the substrate must
/// honor). Float tolerance is the arithmetic's, not a threshold.
pub fn rotation_equivariant(scene: &[Point]) -> f64 {
    let rotated: Vec<Point> = scene.iter().map(|p| rotate90(*p)).collect();
    let original = GeometryField::form(scene);
    let rotated_field = GeometryField::form(&rotated);
    let n = crate::interference_field::N;
    let original_pattern = original.raw_pattern();
    let rotated_pattern = rotated_field.raw_pattern();
    let mut max_err = 0.0f64;
    for y in 0..n {
        for x in 0..n {
            // value at (x, y) in the rotated field equals the original
            // at rot^-1(x, y) = (y, -x).
            let xo = y % n;
            let yo = (n - (x % n)) % n;
            let diff = (rotated_pattern[y * n + x] - original_pattern[yo * n + xo]).abs();
            if diff > max_err {
                max_err = diff;
            }
        }
    }
    max_err
}

/// Qualitative direction: the sign class of an emergent difference
/// (the lattice's own structure — no authored table).
pub fn direction(v: Point) -> (i8, i8) {
    let sign = |c: i64| -> i8 {
        if c > 0 {
            1
        } else if c < 0 {
            -1
        } else {
            0
        }
    };
    (sign(v.0), sign(v.1))
}

/// Direction composition: the sign class of the composed translation
/// (exact vector arithmetic: E + N -> NE).
pub fn compose(direction_ab: Point, direction_bc: Point) -> (i8, i8) {
    direction(min_image((
        direction_ab.0 + direction_bc.0,
        direction_ab.1 + direction_bc.1,
    )))
}

/// Contradiction detection: the same claimed direction for a->b AND for
/// b->a is impossible; opposite claims are consistent.
pub fn contradictory(claimed_ab: (i8, i8), claimed_ba: (i8, i8)) -> bool {
    claimed_ab == claimed_ba && claimed_ab != (0, 0)
}

/// Containment: point-in-triangle by sign-consistency of cross products
/// (integer-exact; scenes posed inside the injective window).
pub fn contains(triangle: &[Point; 3], p: Point) -> bool {
    let (a, b, c) = (triangle[0], triangle[1], triangle[2]);
    let cross = |o: Point, u: Point, v: Point| -> i64 {
        let e1 = min_image((u.0 - o.0, u.1 - o.1));
        let e2 = min_image((v.0 - o.0, v.1 - o.1));
        e1.0 * e2.1 - e1.1 * e2.0
    };
    let d1 = cross(a, b, p);
    let d2 = cross(b, c, p);
    let d3 = cross(c, a, p);
    let has_neg = d1 < 0 || d2 < 0 || d3 < 0;
    let has_pos = d1 > 0 || d2 > 0 || d3 > 0;
    !(has_neg && has_pos)
}
