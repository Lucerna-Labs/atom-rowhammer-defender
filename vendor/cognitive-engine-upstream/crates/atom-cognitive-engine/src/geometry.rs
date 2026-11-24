//! BUILD-8 — the geometric reasoning engine. Precommitted in
//! `docs/BUILD8-GEOMETRY.md` BEFORE this code, under vault Rules 1/2/3
//! and project rules 0/0a/0b.
//!
//! The engine's KNOWLEDGE is emergent: a scene's translation relations
//! (difference vectors) are read from the interference pattern's
//! spectral bins. The QUERIES are the metric's own algebra on that
//! emergent structure — cross-product collinearity, SSS length
//! multisets — containing no scene-specific content (the BUILD-3
//! `delta == 0` precedent).
//!
//! AUDIT: N; min-image canonicalization (torus arithmetic, structural);
//! no scene content, no thresholds.

use crate::interference_field::N;
use crate::weave::Cx;

/// A point on the N-lattice.
pub type Point = (i64, i64);

/// A geometric scene: point-modes whose interference is formed.
pub struct GeometryField {
    pattern: Vec<f64>,
    points: Vec<Point>,
}

impl GeometryField {
    /// Form the scene: each point is a wave at k = point/N
    /// (cycles/site interface, BUILD-1 declaration).
    pub fn form(points: &[Point]) -> Self {
        let mut pattern = vec![0.0f64; N * N];
        for y in 0..N {
            for x in 0..N {
                let mut re = 0.0f64;
                let mut im = 0.0f64;
                for (px, py) in points {
                    let phase = 2.0
                        * core::f64::consts::PI
                        * (*px as f64 * x as f64 + *py as f64 * y as f64)
                        / N as f64;
                    re += phase.cos();
                    im += phase.sin();
                }
                pattern[y * N + x] = re * re + im * im;
            }
        }
        GeometryField {
            pattern,
            points: points.to_vec(),
        }
    }

    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// The formed pattern (readout for frame-equivariance checks).
    pub fn raw_pattern(&self) -> &[f64] {
        &self.pattern
    }

    /// BUILD-16 wiring: rebuild the field from a pattern that travelled
    /// through the working-memory pad. The points remain the task's own
    /// query content (structural).
    pub fn from_pattern(pattern: &[f64], points: &[Point]) -> Self {
        GeometryField {
            pattern: pattern.to_vec(),
            points: points.to_vec(),
        }
    }

    /// The emergent translation relations: the pattern's spectral bins,
    /// min-image canonicalized (the geometry the field actually holds).
    /// DC excluded; conjugates included (a difference and its negation
    /// are both relations present).
    pub fn difference_vectors(&self) -> Vec<Point> {
        let mut out: Vec<Point> = Vec::new();
        for vy in 0..N {
            for vx in 0..N {
                let mut acc = Cx::new(0.0, 0.0);
                for y in 0..N {
                    for x in 0..N {
                        let phase = -2.0
                            * core::f64::consts::PI
                            * (vx as f64 * x as f64 + vy as f64 * y as f64)
                            / N as f64;
                        let value = self.pattern[y * N + x];
                        acc.re += value * phase.cos();
                        acc.im += value * phase.sin();
                    }
                }
                let magnitude = acc.re * acc.re + acc.im * acc.im;
                if magnitude > 1e-9 && !(vx == 0 && vy == 0) {
                    out.push(min_image((vx as i64, vy as i64)));
                }
            }
        }
        out.sort_unstable();
        out.dedup();
        out
    }

    /// QUERY — betweenness: is `probe` the composed translation between
    /// `a` and `b` in the emergent field? (cross product zero AND
    /// same-direction on emergent difference vectors).
    pub fn betweenness(&self, a: Point, b: Point, probe: Point) -> bool {
        let relations = self.difference_vectors();
        let ca = min_image((probe.0 - a.0, probe.1 - a.1));
        let bc = min_image((b.0 - probe.0, b.1 - probe.1));
        if !relations.contains(&ca) || !relations.contains(&bc) {
            return false;
        }
        let cross = ca.0 * bc.1 - ca.1 * bc.0;
        let dot = ca.0 * bc.0 + ca.1 * bc.1;
        cross == 0 && dot > 0
    }

    /// QUERY — SSS congruence of two triangle scenes, from each scene's
    /// emergent side lengths (squared, min-image metric).
    pub fn triangles_congruent(left: &GeometryField, right: &GeometryField) -> bool {
        left.side_lengths_squared() == right.side_lengths_squared()
    }

    /// The emergent side lengths of a three-point scene: the squared
    /// min-image distances of its three pairwise relations.
    pub fn side_lengths_squared(&self) -> Vec<i64> {
        if self.points.len() != 3 {
            return Vec::new();
        }
        let mut lengths = Vec::new();
        for i in 0..3 {
            for j in (i + 1)..3 {
                let d = min_image((
                    self.points[j].0 - self.points[i].0,
                    self.points[j].1 - self.points[i].1,
                ));
                lengths.push(d.0 * d.0 + d.1 * d.1);
            }
        }
        lengths.sort_unstable();
        lengths
    }

    /// The symmetry-derived order: the landscape's argmax-set size
    /// (BUILD-4's k, restated as the scene's symmetry readout).
    pub fn symmetry_order(&self) -> usize {
        let max = self.pattern.iter().copied().fold(0.0f64, f64::max);
        self.pattern
            .iter()
            .filter(|v| (**v - max).abs() < 1e-6 * max)
            .count()
    }
}

/// Torus arithmetic: wrap a vector into the injective window
/// (−N/2, N/2] — structural canonicalization of the lattice.
pub fn min_image(v: Point) -> Point {
    let half = N as i64 / 2;
    let wrap = |c: i64| {
        let m = c.rem_euclid(N as i64);
        if m <= half {
            m
        } else {
            m - N as i64
        }
    };
    (wrap(v.0), wrap(v.1))
}
