//! Reproduction for the Line tool's first point refusing to attach to existing geometry:
//! starting a line on a circle's edge (#11272), on another line's body (#11223), or
//! coincident to a shape (#11291). Drawing *to* a curve already works; only the first
//! point is affected. These drive the same snap path the viewport uses.
#![allow(clippy::unwrap_used)]

use crate::scenario::Harness;
use serde_json::json;

/// A sketch on XY with a circle (centre origin, r 10) and a horizontal line well clear of it,
/// viewed from the top with a fixed camera so the snap tolerances are sub-millimetre.
fn fixture() -> Harness {
    let mut h = Harness::new();
    h.app.preferences.look_at_anim = false;
    for (command, params) in [
        ("sketch.create", json!({"plane": "XY"})),
        ("sketch.circle.center", json!({"center": [0, 0], "radius": 10})),
        ("sketch.line", json!({"points": [[-60, 30], [60, 30]]})),
    ] {
        h.step(&json!({"call": "engine.execute", "params": {"command": command, "params": params}})).unwrap();
    }
    h.step(&json!({"call": "ui.view", "params": {"view": "top"}})).unwrap();
    h.step(&json!({"call": "ui.camera", "params": {"target": [0, 0, 0], "distance": 150}})).unwrap();
    h
}

/// The snap id the viewport would use for the cursor at sketch coordinates `xy`.
fn snap_id(h: &mut Harness, xy: [f64; 2]) -> Option<String> {
    let [x, y] = h.at(&json!({"sketch": xy})).unwrap();
    let pos = egui::pos2(x as f32, y as f32);
    let proj = crate::viewport::projection(&h.app, h.app.viewport.rect.expect("viewport"));
    crate::viewport::sketch_point_at(&h.app, &proj, pos).and_then(|(_, id)| id)
}

#[test]
fn line_first_point_snaps_onto_a_circle_edge() {
    let mut h = fixture();
    h.app.start("sketch.line");
    // Right edge of the circle (radius 10 at the origin). No endpoint, midpoint or crossing
    // is anywhere near, so the only snap that can fire is the on-curve one.
    let id = snap_id(&mut h, [10.0, 0.0]);
    assert!(id.as_deref().is_some_and(|s| s.starts_with("on:")), "first point should snap onto the circle edge, got {id:?}");
}

#[test]
fn line_first_point_snaps_onto_a_line_body() {
    let mut h = fixture();
    h.app.start("sketch.line");
    // On the line (-60,30)-(60,30), 30 mm from both its midpoint and its ends.
    let id = snap_id(&mut h, [-30.0, 30.0]);
    assert!(id.as_deref().is_some_and(|s| s.starts_with("on:")), "first point should snap onto the line body, got {id:?}");
}

#[test]
fn line_first_click_keeps_the_on_curve_constraint() {
    let mut h = fixture();
    h.app.start("sketch.line");
    h.step(&json!({"click": {"sketch": [10, 0]}})).unwrap();
    let first = h.app.tool.as_ref().expect("line tool active").pts.first().and_then(|p| p.1.clone());
    assert!(first.as_deref().is_some_and(|s| s.starts_with("on:")), "placed first point keeps its on-curve argument, got {first:?}");
}
