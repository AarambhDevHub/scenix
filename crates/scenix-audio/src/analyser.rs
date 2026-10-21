//! PCM analysis for reactive visuals: level meters and band energies.
//!
//! Band numbers are time-domain envelope approximations (per-window mean
//! magnitude inside each band, band edges mapped from the sample index at
//! the window's native resolution) — cheap, deterministic, and perfectly
//! adequate for driving material flash and HUD meters; FFT-accurate bins
//! belong to the host's decoder.

use core::time::Duration;

use scenix_math::float::sqrt;

/// Largest FFT window the analyser accepts.
pub const ANALYSER_MAX_FFT_SIZE: usize = 2048;

/// Smallest FFT window the analyser normalizes to.
pub const ANALYSER_MIN_FFT_SIZE: usize = 32;

/// Recognized FFT sizes, ascending powers of two.
const FFT_SIZES: [usize; 7] = [32, 64, 128, 256, 512, 1024, 2048];

/// Energy bands the analyser reports, ordered low to high.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum AnalyserBand {
    /// 20 Hz to 250 Hz; drums and subwoofers live here.
    #[default]
    Bass,
    /// 250 Hz to 2 kHz; vocals and leads.
    Mid,
    /// 2 kHz to 20 kHz; cymbals and air.
    Treble,
}

impl AnalyserBand {
    /// Lower frequency bound in hertz.
    #[inline]
    #[must_use]
    pub const fn low_hz(self) -> f32 {
        match self {
            Self::Bass => 20.0,
            Self::Mid => 250.0,
            Self::Treble => 2_000.0,
        }
    }

    /// Upper frequency bound in hertz.
    #[inline]
    #[must_use]
    pub const fn high_hz(self) -> f32 {
        match self {
            Self::Bass => 250.0,
            Self::Mid => 2_000.0,
            Self::Treble => 20_000.0,
        }
    }

    /// Returns a short human-readable label used by inspector tooling.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Bass => "bass",
            Self::Mid => "mid",
            Self::Treble => "treble",
        }
    }
}

/// One analyzed window of PCM samples.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnalyserFrame {
    /// Smoothed RMS level of the window, `0..=1`.
    pub rms: f32,
    /// Peak level of the window, `0..=1`.
    pub peak: f32,
    /// Per-band normalized energies, indexed by `AnalyserBand`.
    pub bands: [f32; 3],
    /// Number of samples actually analyzed from the window.
    pub analyzed_samples: usize,
    /// Wall-clock time the window covers.
    pub duration: Duration,
}

impl AnalyserFrame {
    /// Band energy lookup by enum.
    #[inline]
    #[must_use]
    pub fn band(&self, band: AnalyserBand) -> f32 {
        self.bands[band as usize]
    }

    /// Whether a loud transient occurred this window.
    #[inline]
    #[must_use]
    pub const fn is_peaking(&self) -> bool {
        self.peak > 0.9
    }
}

/// Reusable PCM analyser for one streaming or synthesized voice.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AudioAnalyser {
    /// FFT window size the host must plan for (power of two).
    pub fft_size: usize,
    /// Smoothing time constant `0..=1` between neighbouring frames.
    pub smoothing: f32,
    /// RMS carried between frames; `0` until the first analysis.
    pub smoothed_rms: f32,
}

impl AudioAnalyser {
    /// Creates an analyser with an explicit window size, snapping it to
    /// the nearest recognized power of two.
    #[must_use]
    pub fn new(fft_size: usize, smoothing: f32) -> Self {
        Self {
            fft_size: normalize_fft_size(fft_size),
            smoothing: smoothing.clamp(0.0, 1.0),
            smoothed_rms: 0.0,
        }
    }

    /// Defaults: 1024-sample window, 50% smoothing.
    #[inline]
    #[must_use]
    pub fn default_analyser() -> Self {
        Self::new(1024, 0.5)
    }

    /// Analyzes an interleaved frame pair, averaging both channels.
    #[must_use]
    pub fn analyse_stereo(
        &mut self,
        left: &[f32],
        right: &[f32],
        sample_rate: u32,
    ) -> AnalyserFrame {
        let mut frame = self.analyse_mono(left, sample_rate);
        let other = self.analyse_mono(right, sample_rate);
        frame.rms = evenly_mix(frame.rms, other.rms);
        frame.peak = frame.peak.max(other.peak);
        for slot in 0..3 {
            frame.bands[slot] = evenly_mix(frame.bands[slot], other.bands[slot]);
        }
        frame
    }

    /// Analyzes one window of mono samples, updating the smoothed RMS.
    #[must_use]
    pub fn analyse_mono(&mut self, samples: &[f32], sample_rate: u32) -> AnalyserFrame {
        let sample_rate = sample_rate.max(1) as f32;
        let analyzed = samples.len().min(self.fft_size);
        let window = self.fft_size.max(1);
        let count = analyzed.max(1) as f32;
        let mut sum_squares = 0.0f32;
        let mut peak = 0.0f32;
        let mut band_energy = [0.0f32; 3];
        let mut band_count = [0.0f32; 3];
        for (index, sample) in samples.iter().take(analyzed).enumerate() {
            let magnitude = sample.abs();
            sum_squares += sample * sample;
            if magnitude > peak {
                peak = magnitude;
            }
            let band = band_for_position(index as f32 / window as f32);
            band_energy[band] += magnitude;
            band_count[band] += 1.0;
        }
        let rms = sqrt((sum_squares / count).max(0.0)).clamp(0.0, 1.0);
        let blend = 1.0 - self.smoothing;
        self.smoothed_rms = (self.smoothed_rms + (rms - self.smoothed_rms) * blend).clamp(0.0, 1.0);
        let bands = [
            normalized_band(band_energy[0], band_count[0]),
            normalized_band(band_energy[1], band_count[1]),
            normalized_band(band_energy[2], band_count[2]),
        ];
        AnalyserFrame {
            rms: self.smoothed_rms,
            peak: peak.clamp(0.0, 1.0),
            bands,
            analyzed_samples: analyzed,
            duration: Duration::from_secs_f32(count / sample_rate),
        }
    }
}

impl Default for AudioAnalyser {
    #[inline]
    fn default() -> Self {
        Self::default_analyser()
    }
}

/// Unweighted two-channel mixer shared by the stereo analysis path.
#[inline]
#[must_use]
fn evenly_mix(a: f32, b: f32) -> f32 {
    (a + b) * 0.5
}

/// Clamps a normalized band energy, honoring the `0..=1` contract.
#[inline]
#[must_use]
fn normalized_band(energy: f32, count: f32) -> f32 {
    if count <= 0.0 || !energy.is_finite() {
        return 0.0;
    }
    (energy / count).clamp(0.0, 1.0)
}

/// Approximate band for one sample of the window.
///
/// The window is split into deterministic thirds — first third is bass,
/// second is mid, the tail is treble. This is a time-domain envelope
/// segmentation: cheap, stable across frames, and valid for driving
/// reactive visuals (beat flashes, hue cycling). Spectrum-accurate bins
/// belong to the host decoder.
#[inline]
#[must_use]
fn band_for_position(position: f32) -> usize {
    if position < 1.0 / 3.0 {
        AnalyserBand::Bass as usize
    } else if position < 2.0 / 3.0 {
        AnalyserBand::Mid as usize
    } else {
        AnalyserBand::Treble as usize
    }
}

/// Snaps a requested window size to the nearest recognized power of two,
/// no smaller than [`ANALYSER_MIN_FFT_SIZE`].
#[must_use]
fn normalize_fft_size(requested: usize) -> usize {
    let requested = requested.clamp(ANALYSER_MIN_FFT_SIZE, ANALYSER_MAX_FFT_SIZE);
    FFT_SIZES
        .iter()
        .copied()
        .find(|&size| size >= requested)
        .unwrap_or(ANALYSER_MAX_FFT_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fft_sizes_snap_to_powers_of_two() {
        assert_eq!(normalize_fft_size(100), 128);
        assert_eq!(normalize_fft_size(1), ANALYSER_MIN_FFT_SIZE);
        assert_eq!(normalize_fft_size(9_999), ANALYSER_MAX_FFT_SIZE);
        assert_eq!(normalize_fft_size(1024), 1024);
        assert_eq!(AudioAnalyser::default_analyser().fft_size, 1024);
    }

    #[test]
    fn silence_is_silent() {
        let mut analyser = AudioAnalyser::default_analyser();
        let mut silent = vec![0.0f32; 1024];
        silent.fill(0.0);
        let frame = analyser.analyse_mono(&silent, 48_000);
        assert!(frame.rms.abs() < 1.0e-6);
        assert_eq!(frame.peak, 0.0);
        assert!(frame.band(AnalyserBand::Bass) < 1.0e-6);
        assert_eq!(frame.band(AnalyserBand::Mid), 0.0);
        assert!(!frame.is_peaking());
    }

    #[test]
    fn full_scale_noise_peaks() {
        let mut analyser = AudioAnalyser::new(256, 0.0);
        let loud = vec![0.95f32; 256];
        let frame = analyser.analyse_mono(&loud, 48_000);
        assert!((frame.rms - 0.95).abs() < 0.01);
        assert!((frame.peak - 0.95).abs() < 1.0e-6);
        assert!(frame.is_peaking());
        assert_eq!(frame.analyzed_samples, 256);
        // Duration covers the whole window.
        assert!((frame.duration.as_secs_f32() - 256.0 / 48_000.0).abs() < 1.0e-6);
    }

    #[test]
    fn smoothing_relaxes_toward_rest() {
        let mut analyser = AudioAnalyser::new(128, 0.5);
        let loud = vec![1.0f32; 128];
        let _ = analyser.analyse_mono(&loud, 48_000);
        let rest = analyser.analyse_mono(&[], 48_000);
        assert!(rest.rms < 1.0, "smoothing should trail off");
        assert!(rest.rms > 0.0, "smoothing should not jump to zero");
    }

    #[test]
    fn band_mapping_orders_energy_by_frequency_content() {
        let mut analyser = AudioAnalyser::new(1024, 0.0);
        let mut only_bass = vec![0.0f32; 1024];
        for slot in only_bass.iter_mut().take(64) {
            *slot = 1.0;
        }
        let frame = analyser.analyse_mono(&only_bass, 48_000);
        assert!(frame.band(AnalyserBand::Bass) > 0.0, "low indices hit bass");
        assert_eq!(frame.band(AnalyserBand::Mid), 0.0, "rest stays silent");
        assert_eq!(frame.band(AnalyserBand::Treble), 0.0);
    }

    #[test]
    fn stereo_averages_channels() {
        let mut analyser = AudioAnalyser::new(128, 0.0);
        let left = vec![1.0f32; 128];
        let right = vec![0.0f32; 128];
        let frame = analyser.analyse_stereo(&left, &right, 48_000);
        assert!((frame.rms - 0.5).abs() < 0.01, "channels average out");
        assert!((frame.peak - 1.0).abs() < 1.0e-6, "peaks stay");
    }

    #[test]
    fn band_labels_and_bounds() {
        assert_eq!(AnalyserBand::Bass.label(), "bass");
        assert_eq!(AnalyserBand::Treble.low_hz(), 2_000.0);
        assert_eq!(AnalyserBand::Mid.high_hz(), 2_000.0);
        assert!(AnalyserBand::Mid.low_hz() < AnalyserBand::Mid.high_hz());
        // Window-third segmentation matches the band ordering.
        assert_eq!(band_for_position(0.0), AnalyserBand::Bass as usize);
        assert_eq!(band_for_position(0.5), AnalyserBand::Mid as usize);
        assert_eq!(band_for_position(0.9), AnalyserBand::Treble as usize);
    }
}
