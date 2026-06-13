//! Source oscillator tests.

use dootdoot_core::{SOURCE_MAX_HARMONICS, source_harmonic_count, source_oscillator_sample};

#[test]
fn source_oscillator_wraps_phase_periodically() {
    for phase in [0.0, 0.125, 0.5, 0.875] {
        assert_eq!(
            source_oscillator_sample(phase, 440.0).to_bits(),
            source_oscillator_sample(phase + 1.0, 440.0).to_bits(),
        );
    }
}

#[test]
fn source_oscillator_outputs_finite_bounded_samples() {
    for frequency_hz in [110.0, 440.0, 2_000.0, 8_000.0] {
        for phase in [0.0, 0.1, 0.25, 0.5, 0.9] {
            let sample = source_oscillator_sample(phase, frequency_hz);

            assert!(sample.is_finite());
            assert!(sample.abs() <= 1.5, "sample {sample} exceeded source bound");
        }
    }
}

#[test]
fn source_oscillator_band_limits_harmonics_against_nyquist() {
    assert_eq!(source_harmonic_count(0.0), 0);
    assert_eq!(source_harmonic_count(110.0), SOURCE_MAX_HARMONICS);
    assert!(source_harmonic_count(8_000.0) < source_harmonic_count(440.0));
    assert_eq!(source_harmonic_count(30_000.0), 0);
}
