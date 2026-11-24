use ram_monitor::{
    row_models::TraceModels, AccessSample, AddressKind, Controller, RowGeometry, Snapshot,
};
use std::time::{Duration, Instant};

fn trace(count: usize, period: usize, step: u64) -> Vec<AccessSample> {
    (0..count)
        .map(|i| AccessSample {
            addr: ((i % period) as u64) << 6,
            time_ns: Some(i as u64 * step),
            address_kind: AddressKind::Synthetic,
            ..Default::default()
        })
        .collect()
}

fn wait_for(controller: &Controller, predicate: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        let snap = controller.snapshot();
        if predicate(&snap) {
            return snap;
        }
        assert!(Instant::now() < end, "last snapshot {snap:?}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn both_disagreements_and_burst_boundary() {
    for (samples, period, step, expect_timing, expect_heat) in [
        (4096, 1, 4000, true, false),
        (8192, 4096, 50, false, true),
        (128, 1, 50, true, false),
        (130, 1, 50, true, true),
        (8192, 8192, 50, false, false),
    ] {
        let mut models = TraceModels::new(RowGeometry::default()).unwrap();
        models.extend(&trace(samples, period, step)).unwrap();
        let r = models.report();
        assert_eq!(r.reading.samples, samples as u64);
        assert_eq!(r.timing_rupture.is_some(), expect_timing);
        assert_eq!(r.reading.first_heat_crossing.is_some(), expect_heat);
        assert_eq!(r.heat_rupture.is_some(), expect_heat);
    }
}

#[test]
fn geometry_provenance_and_missing_time() {
    let mut samples = trace(4, 4, 10);
    samples[1].addr = 1;
    samples[2].addr = 1 << 52;
    samples[3].addr = 1;
    samples[3].pid = 42;
    let mut models = TraceModels::new(RowGeometry::default()).unwrap();
    models.extend(&samples).unwrap();
    assert_eq!(models.report().reading.timing_matches, 1);
    assert_eq!(
        models.extend(&[samples[3], AccessSample::default()]),
        Err("timing and heat models require timestamps")
    );
    assert_eq!(models.report().reading.samples, 4);
    let mut coarse = TraceModels::new(RowGeometry {
        row_shift: 7,
        row_bits: 8,
    })
    .unwrap();
    coarse.extend(&trace(2, 2, 10)).unwrap();
    assert_eq!(coarse.report().reading.timing_matches, 1);
}

#[test]
fn gui_controller_finishes_both_models_past_legacy_stop_and_clears_on_reload() {
    let c = Controller::spawn(|_| Vec::new());
    let mut samples = trace(16_384, 4096, 50);
    samples.extend((0..128).map(|i| AccessSample {
        addr: 0,
        time_ns: Some((16_384 + i) * 50),
        address_kind: AddressKind::Synthetic,
        ..Default::default()
    }));
    let mut direct = TraceModels::new(RowGeometry::default()).unwrap();
    direct.extend(&samples).unwrap();
    let expected = direct.report();
    let n = samples.len() as u64;
    c.load_trace(samples, "two-part".into());
    let staged = wait_for(&c, |s| s.source_name == "two-part");
    assert!(!staged.enabled && staged.model_report.is_none());
    c.set_enabled(true);
    let complete = wait_for(&c, |s| s.halted && s.model_report == Some(Ok(expected)));
    assert!(expected.timing_rupture.unwrap() > complete.samples);
    assert_eq!(expected.reading.samples, n);
    c.set_enabled(false);
    wait_for(&c, |s| !s.enabled);
    c.load_trace(trace(256, 256, 1), "unique".into());
    assert!(wait_for(&c, |s| s.source_name == "unique")
        .model_report
        .is_none());
    c.set_enabled(true);
    let unique = wait_for(&c, |s| {
        s.model_report
            .is_some_and(|r| r.is_ok_and(|m| m.reading.samples == 256))
    });
    let result = unique.model_report.unwrap().unwrap();
    assert!(result.timing_rupture.is_none() && result.reading.first_heat_crossing.is_none());
    c.reset();
    assert!(wait_for(&c, |s| !s.enabled).model_report.is_none());
}

#[test]
fn switching_off_stops_comparison_and_apply_restarts_from_fresh_geometry() {
    let c = Controller::spawn(|_| Vec::new());
    c.load_trace(trace(98_304, 4096, 50), "long".into());
    c.set_enabled(true);
    wait_for(&c, |s| {
        s.model_report
            .is_some_and(|r| r.is_ok_and(|m| m.reading.samples > 0 && m.reading.samples < 98_304))
    });
    c.set_enabled(false);
    let paused = wait_for(&c, |s| !s.enabled).model_report;
    assert!(paused.unwrap().unwrap().reading.samples < 98_304);
    std::thread::sleep(Duration::from_millis(150));
    assert_eq!(c.snapshot().model_report, paused);
    c.apply(ram_monitor::Settings {
        row_shift: 7,
        ..Default::default()
    });
    wait_for(&c, |s| s.model_report.is_none());
    c.set_enabled(true);
    let completed = wait_for(&c, |s| {
        s.model_report
            .is_some_and(|r| r.is_ok_and(|m| m.reading.samples == 98_304))
    });
    assert_eq!(
        completed
            .model_report
            .unwrap()
            .unwrap()
            .reading
            .timing_matches,
        49_152
    );
}
