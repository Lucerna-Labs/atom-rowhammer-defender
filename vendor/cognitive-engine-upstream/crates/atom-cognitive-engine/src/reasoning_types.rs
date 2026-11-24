//! BUILD-10 — the reasoning-type suite. Precommitted in
//! `docs/BUILD10-REASONING-TYPES.md` BEFORE this code, under vault
//! Rules 1/2/3 and project rules 0/0a/0b.
//!
//! AUDIT: structural constants only (N, lattice structures, the
//! verified operators of earlier builds). Queries carry no scene
//! content.

use crate::geometry::{min_image, GeometryField, Point};
use crate::interference_field::N;
use crate::weave::Cx;

// ---------- RT1: bottom-up composition ----------

/// Part supports and the union scene's support: the new bins are the
/// cross-part relations (the interface structure).
pub fn bottom_up(part1: &[Point], part2: &[Point]) -> (usize, usize, usize, usize) {
    let s1 = GeometryField::form(part1).difference_vectors().len();
    let s2 = GeometryField::form(part2).difference_vectors().len();
    let mut whole = part1.to_vec();
    whole.extend_from_slice(part2);
    let sw = GeometryField::form(&whole).difference_vectors().len();
    (s1, s2, sw, sw - s1 - s2)
}

// ---------- RT2: top-down constraint ----------

fn closure_size(differences: &[Point]) -> usize {
    let modulus = N as i64;
    let mut reachable: Vec<Point> = vec![(0, 0)];
    let mut frontier = vec![(0i64, 0i64)];
    while let Some((x, y)) = frontier.pop() {
        for (gx, gy) in differences {
            let next = ((x + gx).rem_euclid(modulus), (y + gy).rem_euclid(modulus));
            if !reachable.contains(&next) {
                reachable.push(next);
                frontier.push(next);
            }
        }
    }
    reachable.len()
}

fn scene_differences(points: &[Point]) -> Vec<Point> {
    let mut out = Vec::new();
    for i in 0..points.len() {
        for j in (i + 1)..points.len() {
            let d = min_image((points[j].0 - points[i].0, points[j].1 - points[i].1));
            out.push((d.0.rem_euclid(N as i64), d.1.rem_euclid(N as i64)));
        }
    }
    out
}

/// Top-down: the whole's closure size and whether a part's differences
/// are inside it (the constraint the whole places on the parts).
pub fn top_down(part: &[Point], whole: &[Point]) -> (usize, usize, bool) {
    let whole_closure = closure_size(&scene_differences(whole));
    let part_closure = closure_size(&scene_differences(part));
    let inside = scene_differences(part).iter().all(|d| {
        let mut reachable: Vec<Point> = vec![(0, 0)];
        let mut frontier = vec![(0i64, 0i64)];
        let modulus = N as i64;
        while let Some((x, y)) = frontier.pop() {
            for (gx, gy) in scene_differences(whole) {
                let next = ((x + gx).rem_euclid(modulus), (y + gy).rem_euclid(modulus));
                if !reachable.contains(&next) {
                    reachable.push(next);
                    frontier.push(next);
                }
            }
        }
        reachable.contains(&(d.0.rem_euclid(modulus), d.1.rem_euclid(modulus)))
    });
    (whole_closure, part_closure, inside)
}

/// The complement check: closure(part1 ∪ part2) == closure(whole).
pub fn top_down_complement(part1: &[Point], part2: &[Point], whole: &[Point]) -> bool {
    let mut both = scene_differences(part1);
    both.extend(scene_differences(part2));
    closure_size(&both) == closure_size(&scene_differences(whole))
}

// ---------- RT3: abduction + counterfactual ----------

/// Abduction: recover the world's hidden wave from an action and a
/// difference relation observed to contain (action − W).
pub fn recover_world(action: Point, observed_difference: Point) -> Point {
    (
        (action.0 - observed_difference.0).rem_euclid(N as i64),
        (action.1 - observed_difference.1).rem_euclid(N as i64),
    )
}

/// Counterfactual: the differences a DIFFERENT action would have
/// produced, from the recovered world — to be verified when taken.
pub fn counterfactual_differences(action: Point, world: Point) -> Point {
    min_image((action.0 - world.0, action.1 - world.1))
}

// ---------- RT4: analogy ----------

/// Translation analogy: transfer the relation A→B onto C.
pub fn analogical_transfer(a: Point, b: Point, c: Point) -> Point {
    let relation = (b.0 - a.0, b.1 - a.1);
    (c.0 + relation.0, c.1 + relation.1)
}

/// Rotational analogy: given a scene and a rotated copy, recover which
/// grid rotation maps them (by side-length + chirality signature), then
/// apply it to a new point. Returns None if no rotation matches.
pub fn analogical_rotation(base: &[Point], copy: &[Point]) -> Option<usize> {
    // FIRST VERSION FAILED AND IS KEPT ON RECORD: side-length signatures
    // cannot identify WHICH rotation (every rotation preserves lengths —
    // the same blindness as SSS). The distinguishing signature is the
    // set of SIGNED pairwise difference VECTORS, which rotate with R.
    let signature = |points: &[Point]| -> Vec<Point> {
        let mut vectors = Vec::new();
        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                vectors.push(min_image((points[j].0 - points[i].0, points[j].1 - points[i].1)));
            }
        }
        vectors.sort_unstable();
        vectors
    };
    let copy_sig = signature(copy);
    for index in 0..4usize {
        let mapped: Vec<Point> = base.iter().map(|p| apply_rotation(index, *p)).collect();
        if signature(&mapped) == copy_sig {
            return Some(index);
        }
    }
    None
}

/// Apply a recovered rotation index to a point.
pub fn apply_rotation(index: usize, p: Point) -> Point {
    use crate::spatial::{rotate180, rotate270, rotate90};
    match index {
        0 => p,
        1 => rotate90(p),
        2 => rotate180(p),
        _ => rotate270(p),
    }
}

// ---------- RT5: defeasible (coherent retraction) ----------

/// Coherent amplitude memory: contributions SUM as amplitudes, so any
/// epoch is exactly retractable by adding its negated-phase
/// contribution (commutative undo). The BUILD-1 G3 trade applies:
/// coherence carries phantom cross-epoch modes.
pub struct CoherentMemory {
    field: Vec<Cx>,
    contributions: Vec<Vec<Cx>>,
}

impl CoherentMemory {
    pub fn new() -> Self {
        CoherentMemory {
            field: vec![Cx::new(0.0, 0.0); N * N],
            contributions: Vec::new(),
        }
    }

    /// Present a scene (as point-waves) at the given phase.
    pub fn present(&mut self, points: &[Point], phase: f64) {
        let carrier = Cx::new(phase.cos(), phase.sin());
        let mut contribution = vec![Cx::new(0.0, 0.0); N * N];
        for y in 0..N {
            for x in 0..N {
                let mut re = 0.0f64;
                let mut im = 0.0f64;
                for (px, py) in points {
                    let angle = 2.0
                        * core::f64::consts::PI
                        * (*px as f64 * x as f64 + *py as f64 * y as f64)
                        / N as f64;
                    re += angle.cos();
                    im += angle.sin();
                }
                contribution[y * N + x] = Cx::new(re * carrier.re - im * carrier.im, re * carrier.im + im * carrier.re);
            }
        }
        self.apply(&contribution, 1.0);
        self.contributions.push(contribution);
    }

    fn apply(&mut self, contribution: &[Cx], sign: f64) {
        for (site, c) in self.field.iter_mut().zip(contribution) {
            site.re += sign * c.re;
            site.im += sign * c.im;
        }
    }

    /// Retract the epoch at `index` EXACTLY: add the negated phase of
    /// its contribution (commutative — any epoch, any order).
    pub fn retract(&mut self, index: usize) {
        if let Some(contribution) = self.contributions.get(index).cloned() {
            self.apply(&contribution, -1.0);
        }
    }

    /// Restore a retracted epoch.
    pub fn restore(&mut self, index: usize) {
        if let Some(contribution) = self.contributions.get(index).cloned() {
            self.apply(&contribution, 1.0);
        }
    }

    /// Support of the coherent field (integer bin count).
    pub fn support(&self) -> usize {
        let intensity: Vec<f64> = self
            .field
            .iter()
            .map(|v| v.re * v.re + v.im * v.im)
            .collect();
        let mut spectrum = Vec::with_capacity(N * N);
        for vy in 0..N {
            for vx in 0..N {
                let mut acc = 0.0f64;
                for y in 0..N {
                    for x in 0..N {
                        let phase = -2.0
                            * core::f64::consts::PI
                            * (vx as f64 * x as f64 + vy as f64 * y as f64)
                            / N as f64;
                        acc += intensity[y * N + x] * phase.cos();
                    }
                }
                spectrum.push(acc * acc);
            }
        }
        let max = spectrum.iter().copied().fold(0.0f64, f64::max);
        if max <= 0.0 {
            return 0;
        }
        spectrum.iter().filter(|v| **v > max * 1e-18).count()
    }
}

impl Default for CoherentMemory {
    fn default() -> Self {
        Self::new()
    }
}
