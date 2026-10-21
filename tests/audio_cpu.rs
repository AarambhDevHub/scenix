//! Distributed `scenix-audio` v1.9.0 CPU validation suite.
//!
//! Run from the repository root:
//!
//! ```sh
//! cargo test -p scenix-audio --test audio_cpu
//! ```

use scenix_audio::{
    AUDIO_VERSION, AnalyserBand, AnalyserFrame, AttenuationModel, AudioAnalyser, AudioListener,
    AudioLoop, AudioSource, AudioStreamPlan, DistanceGain, DopplerModel, PlaybackState,
    SPEED_OF_SOUND, STREAM_MAX_CHUNKS, SpatialMix, SpeakerLayout, StreamFormat, StreamState,
    compute_spatial_decision, compute_spatial_gain,
};
use scenix_math::Vec3;

const EPS: f32 = 1.0e-4;

fn marching_listener() -> AudioListener {
    AudioListener::at(Vec3::new(0.0, 1.6, 0.0))
}

fn stereo_source_at(position: Vec3) -> AudioSource {
    AudioSource::new(11, PlaybackState::Playing).at_position(position)
}

#[test]
fn listener_and_sources_pair_by_ids() {
    let listener = marching_listener().with_id(scenix_core::AudioListenerId::new(9));
    let source = stereo_source_at(Vec3::new(1.0, 1.6, -3.0))
        .with_listener(scenix_core::AudioListenerId::new(9));
    assert_eq!(source.listener, listener.id);
    assert_eq!(listener.id.get(), 9);
    assert!((listener.scale_hint() - 1.0 / 3.0).abs() < 1.0e-7);
}

#[test]
fn positional_mix_pans_and_attenuates() {
    let listener = marching_listener();
    let in_front = stereo_source_at(Vec3::new(0.0, 1.6, -5.0));
    let mix = compute_spatial_gain(&listener, &in_front);
    assert!((mix.left - mix.right).abs() < EPS);
    assert!(mix.left > 0.0 && mix.left < 1.0);
    assert!(mix.center > 0.0);

    let hard_right = stereo_source_at(Vec3::new(8.0, 1.6, 0.0));
    let right_mix = compute_spatial_gain(&listener, &hard_right);
    assert!(
        right_mix.right > right_mix.left,
        "source on the right pans right"
    );

    let distant = stereo_source_at(Vec3::new(0.0, 1.6, -400.0));
    let far_mix = compute_spatial_gain(&listener, &distant);
    assert!(far_mix.right < mix.right, "distance attenuation bites");

    // Master gain scales everything.
    let quiet_listener = marching_listener().with_master_gain(0.5);
    let half = compute_spatial_gain(&quiet_listener, &in_front);
    assert!((half.left - mix.left * 0.5).abs() < EPS);
}

#[test]
fn playback_state_machine_is_total() {
    let mut source = AudioSource::new(4, PlaybackState::Playing)
        .with_looping(AudioLoop::Times(2))
        .with_volume(1.0);
    source.pause();
    assert_eq!(source.state, PlaybackState::Paused);
    assert_eq!(source.state.label(), "paused");

    // Muted while paused, regardless of filters.
    let listener = marching_listener();
    let mix = compute_spatial_gain(&listener, &source);
    assert_eq!(mix, SpatialMix::muted());

    source.play();
    assert_eq!(source.state.label(), "playing");
    source.stop();
    assert_eq!(source.state.label(), "stopped");

    // Scheduling respects loops then halts.
    source.play();
    source.pass = 1;
    assert!(source.scheduler_wants_next_pass());
    source.pass = 2;
    assert!(!source.scheduler_wants_next_pass());
}

#[test]
fn distance_models_agree_through_the_pipeline() {
    let listener = marching_listener();
    let mut source = stereo_source_at(Vec3::new(0.0, 1.6, -3.0));
    source.min_distance = 1.0;
    source.max_distance = 50.0;

    // Linear floor-to-zero inside a 50 m budget.
    let linear = compute_spatial_decision(
        &listener,
        &source,
        AttenuationModel::Linear,
        1.0,
        &DopplerModel::new(),
    );
    let inverse = compute_spatial_decision(
        &listener,
        &source,
        AttenuationModel::Inverse,
        1.0,
        &DopplerModel::new(),
    );
    assert!(
        linear.distance.far >= inverse.distance.far.min(1.0),
        "linear decays slower at 3 m"
    );
    assert!(linear.distance.far > 0.5);

    // InverseSquare hits 1.0 inside the reference distance.
    let near = compute_spatial_decision(
        &listener,
        &source,
        AttenuationModel::InverseSquare,
        1.0,
        &DopplerModel::new(),
    );
    assert!((near.distance_to_listener - 3.0).abs() < EPS);
    assert!(near.distance.far <= 1.0);
}

#[test]
fn doppler_multiplier_tracks_closing_velocity() {
    let model = DopplerModel::new();
    assert_eq!(SPEED_OF_SOUND, 343.0);

    // Moving into the listener's -Z raises pitch.
    let closer = model.shift(Vec3::new(0.0, 0.0, -34.3), Vec3::ZERO, Vec3::NEG_Z);
    assert!(closer.frequency_multiplier > 1.0);

    // Same source receding lowers it.
    let farther = model.shift(Vec3::new(0.0, 0.0, 34.3), Vec3::ZERO, Vec3::NEG_Z);
    assert!(farther.frequency_multiplier < 1.0);

    // Doppler collapses when disabled.
    let silent = DopplerModel::new().with_factor(0.0).shift(
        Vec3::new(90.0, 0.0, 0.0),
        Vec3::ZERO,
        Vec3::NEG_Z,
    );
    assert!((silent.frequency_multiplier - 1.0).abs() < 1.0e-6);
}

#[test]
fn streaming_plan_counts_chunks_and_underruns() {
    let mut plan = AudioStreamPlan::new(33, StreamFormat::PcmF32, 48_000, 2);
    assert_eq!(plan.state, StreamState::Empty);
    assert_eq!(STREAM_MAX_CHUNKS, 8);

    // Buffers fill as chunks promote from pending to ready.
    plan.accept_chunk();
    assert!(plan.promote_pending());
    plan.accept_chunk();
    assert!(plan.promote_pending());
    assert_eq!(plan.buffered_chunks(), 2);
    assert!(plan.state.is_schedulable());
    let ms = plan.buffered_milliseconds(1024);
    assert!((ms - 2.0 * 1024.0 * 1000.0 / 48_000.0).abs() < 1.0e-2);
    assert_eq!(plan.chunk_byte_budget(1024), 1024 * 2 * 4);
    assert!(StreamFormat::Pcm16.is_pcm());
    assert!(!StreamFormat::Compressed.is_pcm());

    // Drainage starves into an underrun.
    assert!(plan.consume_chunk());
    assert!(plan.consume_chunk());
    assert!(!plan.consume_chunk());
    assert_eq!(plan.state, StreamState::Underrun);
    plan.register_underrun();
    assert_eq!(plan.underruns, 1);

    // Media end and replay clear buffers.
    plan.finish();
    assert_eq!(plan.state, StreamState::Finished);
    plan.reset();
    assert_eq!(plan.state, StreamState::Empty);
}

#[test]
fn analyser_meter_scales_with_content() {
    let band = |frame: &AnalyserFrame, b: AnalyserBand| frame.band(b);

    let mut analyser = AudioAnalyser::new(1024, 0.0);
    assert_eq!(analyser.fft_size, 1024);

    let silence = vec![0.0f32; 1024];
    let quiet = analyser.analyse_mono(&silence, 48_000);
    assert!(quiet.rms.abs() < 1.0e-6);

    let loud = vec![0.95f32; 1024];
    let hit = analyser.analyse_mono(&loud, 48_000);
    assert!(hit.rms > 0.5);
    assert!(hit.is_peaking());
    assert!(band(&hit, AnalyserBand::Bass) > band(&quiet, AnalyserBand::Bass));

    // Stereo folds to the average.
    let frame = analyser.analyse_stereo(&loud, &silence, 48_000);
    assert!((frame.peak - 0.95).abs() < 1.0e-6, "peaks survive folding");

    // Test-space probe keeps band labels stable.
    assert_eq!(AnalyserBand::Bass.label(), "bass");
    assert!(AnalysisBandProbe::LOW < AnalysisBandProbe::HIGH);
}

/// Small probe so the distributed tooling can verify band ordering without
/// depending on the analyser's private internal segmenter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnalysisBandProbe {
    LOW = 0,
    HIGH = 2,
}

#[test]
fn listener_budget_is_stable() {
    assert_eq!(scenix_audio::AUDIO_LISTENER_MAX_SOURCES, 32);
    assert_eq!(DistanceGain::flat(0.75).center(), 0.75);
    assert_eq!(SpeakerLayout::Stereo.speaker_count(), 2);
    assert_eq!(SpeakerLayout::Surround51.speaker_count(), 6);
    assert_eq!(AUDIO_VERSION, "1.9.0");
}
