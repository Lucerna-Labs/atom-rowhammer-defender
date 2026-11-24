//! BUILD-9 gates — precommitted in docs/BUILD9-SPATIAL.md BEFORE this
//! code. Orientation, frames, directions: the layer the metric cannot
//! see (SSS is blind to handedness; this is not).

use atom_cognitive_engine::geometry::GeometryField;
use atom_cognitive_engine::spatial::{
    compose, contains, contradictory, direction, handedness, reflect, rotate180, rotate270,
    rotate90, rotation_equivariant, signed_area,
};

/// SP1 (precommitted: pass) — CHIRALITY: rotation preserves handedness,
/// reflection flips it, side lengths are identical throughout (the
/// metric cannot distinguish; the orientation query can).
#[test]
fn gate_sp1_chirality() {
    let t = [(2i64, 2i64), (6, 2), (2, 5)];
    let t90 = [rotate90(t[0]), rotate90(t[1]), rotate90(t[2])];
    let t180 = [rotate180(t[0]), rotate180(t[1]), rotate180(t[2])];
    let t270 = [rotate270(t[0]), rotate270(t[1]), rotate270(t[2])];
    let tf = [reflect(t[0]), reflect(t[1]), reflect(t[2])];
    eprintln!(
        "signed areas: base {}, rot90 {}, rot180 {}, rot270 {}, reflected {}",
        signed_area(&t), signed_area(&t90), signed_area(&t180), signed_area(&t270), signed_area(&tf)
    );
    assert_eq!(signed_area(&t), 12);
    assert_eq!(handedness(&t), 1);
    assert_eq!(handedness(&t90), 1, "rotation preserves handedness");
    assert_eq!(handedness(&t180), 1);
    assert_eq!(handedness(&t270), 1);
    assert_eq!(handedness(&tf), -1, "reflection flips handedness");
    // The metric is blind: identical side lengths for both.
    let sides = |tri: &[(i64, i64); 3]| {
        let mut v = Vec::new();
        for i in 0..3 {
            for j in (i + 1)..3 {
                let d = (tri[j].0 - tri[i].0, tri[j].1 - tri[i].1);
                v.push(d.0 * d.0 + d.1 * d.1);
            }
        }
        v.sort_unstable();
        v
    };
    assert_eq!(sides(&t), sides(&tf), "SSS blind to handedness");
}

/// SP2 (precommitted: pass) — FRAME EQUIVARIANCE: the field of a
/// rotated scene equals the rotated field (float-exact), and BUILD-8
/// judgments commute with frame change.
#[test]
fn gate_sp2_frame_equivariance() {
    let scene = [(3i64, 1i64), (5, 2), (4, 4)];
    let err90 = rotation_equivariant(&scene);
    eprintln!("rotation equivariance max err: {err90:.3e}");
    assert!(err90 < 1e-9, "90-degree frame equivariance");

    // Betweenness commutes with the frame change.
    let base = GeometryField::form(&[(2, 2), (6, 2), (4, 2)]);
    let rotated_points: Vec<(i64, i64)> = [(2i64, 2i64), (6, 2), (4, 2)]
        .iter()
        .map(|p| rotate90(*p))
        .collect();
    let rotated = GeometryField::form(&rotated_points);
    assert!(base.betweenness((2, 2), (6, 2), (4, 2)));
    assert!(
        rotated.betweenness(rotated_points[0], rotated_points[1], rotated_points[2]),
        "betweenness commutes with the frame"
    );

    // Congruence survives the frame change (rotated copy still congruent).
    let t1 = GeometryField::form(&[(2, 2), (6, 2), (2, 5)]);
    let t2 = GeometryField::form(&[(10, 10), (10, 14), (7, 10)]);
    assert!(GeometryField::triangles_congruent(&t1, &t2));
}

/// SP3 (precommitted: pass) — DIRECTION COMPOSITION and contradiction
/// detection from the lattice's own sign classes.
#[test]
fn gate_sp3_directions() {
    let east = (4i64, 0i64);
    let north = (0i64, 3i64);
    eprintln!(
        "E+N = {:?}; E+E = {:?}; contradiction(E-claim on both sides) {}; opposite claims {}",
        compose(east, north),
        compose(east, east),
        contradictory(direction(east), direction(east)),
        contradictory(direction(east), direction((-4, 0)))
    );
    assert_eq!(direction(east), (1, 0));
    assert_eq!(direction(north), (0, 1));
    assert_eq!(compose(east, north), (1, 1), "E + N -> NE");
    assert_eq!(compose(east, east), (1, 0), "E + E -> E");
    assert!(contradictory(direction(east), direction(east)), "same dir claimed both ways");
    assert!(
        !contradictory(direction(east), direction((-4, 0))),
        "opposite claims are consistent"
    );
}

/// SP4 (precommitted: pass) — CONTAINMENT: integer-exact point-in-
/// triangle, and frame-invariant.
#[test]
fn gate_sp4_containment() {
    let tri = [(2i64, 2i64), (10, 2), (2, 10)];
    assert!(contains(&tri, (4, 4)), "inside");
    assert!(!contains(&tri, (9, 9)), "outside");
    assert!(!contains(&tri, (1, 9)), "outside");
    // Frame change preserves containment.
    let rot = [rotate90(tri[0]), rotate90(tri[1]), rotate90(tri[2])];
    let p_rot = rotate90((4, 4));
    eprintln!("rotated frame: inside {}", contains(&rot, p_rot));
    assert!(contains(&rot, p_rot), "containment commutes with the frame");
}

/// SP5 (precommitted: pass) — AUDIT: queries take only points; no scene
/// content; the same code judges every scene. Variants: 180/270 also
/// equivariant.
#[test]
fn gate_sp5_audit_and_variants() {
    assert!(rotation_equivariant(&[(3, 1), (5, 2)]) < 1e-9);
    // rotate180-equivariance via composition of the 90-degree check:
    let scene = [(3i64, 1i64), (5, 2)];
    let rot180: Vec<(i64, i64)> = scene.iter().map(|p| rotate180(*p)).collect();
    let err = {
        let a = GeometryField::form(&scene);
        let b = GeometryField::form(&rot180);
        let n = 20usize;
        let mut e = 0.0f64;
        for y in 0..n {
            for x in 0..n {
                // CORRECTED mapping, kept on record: 180-degree maps
                // site (x, y) to (-x, -y) = ((N-x)%N, (N-y)%N) — NOT
                // (x+10, y+10) (equal only at two sites; my first-run
                // mapping error measured 4.0).
                let d = (a.raw_pattern()[y * n + x]
                    - b.raw_pattern()[((n - y) % n) * n + ((n - x) % n)])
                    .abs();
                if d > e {
                    e = d;
                }
            }
        }
        e
    };
    eprintln!("180-degree equivariance max err: {err:.3e}");
    assert!(err < 1e-9);
}
