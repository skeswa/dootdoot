//! Audio sample quantization tests.

use dootdoot_core::{PCM_I16_SCALE, quantize_sample};

#[test]
fn quantization_maps_full_scale_audio_to_i16_bounds() {
    assert_eq!(quantize_sample(1.0), i16::MAX);
    assert_eq!(quantize_sample(-1.0), i16::MIN);
    assert_eq!(quantize_sample(2.0), i16::MAX);
    assert_eq!(quantize_sample(-2.0), i16::MIN);
}

#[test]
fn quantization_uses_round_half_to_even_without_dither() {
    assert_eq!(quantize_sample(0.5 / PCM_I16_SCALE), 0);
    assert_eq!(quantize_sample(1.5 / PCM_I16_SCALE), 2);
    assert_eq!(quantize_sample(2.5 / PCM_I16_SCALE), 2);
    assert_eq!(quantize_sample(-0.5 / PCM_I16_SCALE), 0);
    assert_eq!(quantize_sample(-1.5 / PCM_I16_SCALE), -2);
    assert_eq!(quantize_sample(-2.5 / PCM_I16_SCALE), -2);
}

#[test]
fn quantization_handles_non_finite_samples_deterministically() {
    assert_eq!(quantize_sample(f64::NAN), 0);
    assert_eq!(quantize_sample(f64::INFINITY), i16::MAX);
    assert_eq!(quantize_sample(f64::NEG_INFINITY), i16::MIN);
}
