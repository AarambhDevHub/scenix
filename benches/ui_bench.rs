use std::hint::black_box;
use std::time::Instant;

use scenix_math::Vec2;
use scenix_ui::{
    Anchor, BudgetPolicy, DockKind, Edges, FrameSample, PanelId, PanelSpec, PanelStack,
    RendererStats, compose_panel_overlay, panel_placement, safe_rect,
};

fn bench(name: &str, iterations: usize, mut f: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    println!("{name}: {iterations} iterations in {elapsed:?}");
}

fn main() {
    // Rolling statistics: the fixed-charge ring plus derive pass, the
    // straight-line hot loop an overlay banner hits every frame.
    let mut stats = RendererStats::new();
    bench("stats_record_120_frames", 2_000, || {
        for frame in 0..120 {
            stats.record_frame(FrameSample::sanitized(16.7, 640, 900_000, 96 * 1024));
            black_box(&stats);
        }
    });
    bench("stats_mean_fps", 50_000, || {
        black_box(stats.fps());
        black_box(stats.summary_line());
        black_box(stats.summary_counts());
    });

    // Layout: nine-way anchor solve and safe-rect arithmetic, allocation free.
    let viewport = Vec2::new(1920.0, 1080.0);
    let inset = Edges::uniform(24.0);
    bench("anchor_place", 200_000, || {
        black_box(Anchor::CenterRight.place(
            black_box(viewport),
            inset,
            Vec2::new(240.0, 64.0),
            Vec2::new(8.0, 0.0),
        ));
        black_box(safe_rect(black_box(viewport), inset));
    });

    // Dock solve: a full editor shell repricing every frame.
    let mut stack = PanelStack::new();
    stack.upsert(PanelSpec::docked(
        PanelId(0),
        "Scene Tree".into(),
        DockKind::Left,
        320.0,
    ));
    stack.upsert(PanelSpec::docked(
        PanelId(1),
        "Inspector".into(),
        DockKind::Left,
        360.0,
    ));
    stack.upsert(PanelSpec::docked(
        PanelId(2),
        "Assets".into(),
        DockKind::Right,
        300.0,
    ));
    stack.upsert(PanelSpec::docked(
        PanelId(3),
        "Timeline".into(),
        DockKind::Bottom,
        220.0,
    ));
    stack.upsert(PanelSpec::docked(
        PanelId(4),
        "Material".into(),
        DockKind::Right,
        260.0,
    ));
    bench("panel_placement", 50_000, || {
        black_box(panel_placement(black_box(&stack), black_box(viewport)));
    });

    // Overlay composition: backdrops plus stats banner text per frame.
    bench("compose_overlay", 20_000, || {
        let frame = compose_panel_overlay(black_box(viewport), black_box(&stack), Some(&stats));
        black_box(frame);
    });

    // Budget policy check on a fully-loaded window.
    bench("budget_check", 20_000, || {
        bench_with_stats(&stats);
    });

    let _ = BudgetPolicy::SIXTY_HZ;
}

fn bench_with_stats(stats: &RendererStats) {
    let mut warnings = Vec::new();
    stats.budget_warnings(&BudgetPolicy::SIXTY_HZ, &mut warnings);
    black_box(warnings.len());
}
