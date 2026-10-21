//! Positional audio mixing and metering in deterministic CPU code.
//!
//! Faces a listener down-range from a room corner, decides the spatial mix
//! for one playing source, checks stream buffering health, and meters a
//! 1 kHz sine through the analyser. Run with:
//!
//! `cargo run -p scenix --example audio_positions --features audio`

use scenix::{
    AttenuationModel, AudioAnalyser, AudioListener, AudioSource, AudioStreamPlan, DopplerModel,
    PlaybackState, SPEED_OF_SOUND, StreamFormat, Vec3, compute_spatial_decision,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Listener stands two meters back, looking down-range (-Z).
    let listener = AudioListener::axes(Vec3::new(0.0, 1.5, 2.0), Vec3::NEG_Z, Vec3::Y);

    // A source plays three meters ahead and one left of the view axis.
    let mut source = AudioSource::new(1, PlaybackState::Playing);
    source.position = Vec3::new(-1.0, 1.0, -1.0);

    let decision = compute_spatial_decision(
        &listener,
        &source,
        AttenuationModel::Inverse,
        1.0,
        &DopplerModel::default(),
    );
    assert!(decision.audible, "an in-range playing source is audible");
    let left = decision.left_gain();
    let right = decision.right_gain();
    println!(
        "mix: distance={:.2} m left={:.3} right={:.3} doppler={:.4}",
        decision.distance_to_listener, left, right, decision.frequency_multiplier,
    );
    assert!(left > right, "the source sits left of the view axis");

    // A fast passing source compresses the re-playback pitch.
    let mut mover = AudioSource::new(2, PlaybackState::Playing);
    mover.position = Vec3::new(0.0, 1.0, -2.0);
    mover.velocity = Vec3::new(0.0, 0.0, 34.3);
    let fast = compute_spatial_decision(
        &listener,
        &mover,
        AttenuationModel::Inverse,
        1.0,
        &DopplerModel::default(),
    );
    println!(
        "closing fly-by: radial={:+.2} m/s multiplier={:.4} (speed of sound={:.0})",
        fast.radial_velocity, fast.frequency_multiplier, SPEED_OF_SOUND,
    );
    assert!(fast.frequency_multiplier > 1.0, "closing sources pitch up");

    // Buffered chunk health math for a 48 kHz stereo float stream.
    let mut plan = AudioStreamPlan::new(4, StreamFormat::PcmF32, 48_000, 2);
    plan.accept_chunk();
    plan.accept_chunk();
    assert!(plan.promote_pending());
    assert!(plan.promote_pending());
    assert_eq!(plan.ready_chunks, 2);
    let ms = plan.buffered_milliseconds(1024);
    println!(
        "buffer: ready={} chunks = {:.2} ms of audio",
        plan.ready_chunks, ms
    );

    // The analyser bands a full-scale sine into the mid range and peaks.
    let mut analyser = AudioAnalyser::new(1024, 0.0);
    let sine: Vec<f32> = (0..1024)
        .map(|i| (core::f32::consts::TAU * 1000.0 * i as f32 / 48_000.0).sin())
        .collect();
    let frame = analyser.analyse_mono(&sine, 48_000);
    println!(
        "meter: rms={:.3} peak={:.3} band_energies=({:.3},{:.3},{:.3})",
        frame.rms,
        frame.peak,
        frame.band(scenix::AnalyserBand::Bass),
        frame.band(scenix::AnalyserBand::Mid),
        frame.band(scenix::AnalyserBand::Treble),
    );
    assert!(frame.peak > 0.9, "a full-scale sine peaks the meter");
    assert!(
        frame.band(scenix::AnalyserBand::Mid) >= frame.band(scenix::AnalyserBand::Bass),
        "1 kHz energy lands in the mid band"
    );
    Ok(())
}
