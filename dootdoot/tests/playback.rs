//! Playback conversion tests.

use dootdoot::playback_samples;

#[test]
fn playback_samples_convert_i16_pcm_to_normalized_float_range() {
    let samples = playback_samples(&[i16::MIN, 0, i16::MAX]);

    assert_eq!(samples[0].to_bits(), (-1.0_f32).to_bits());
    assert_eq!(samples[1].to_bits(), 0.0_f32.to_bits());
    assert_eq!(
        samples[2].to_bits(),
        (f32::from(i16::MAX) / 32_768.0_f32).to_bits(),
    );
}
