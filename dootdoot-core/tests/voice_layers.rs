//! Layered droid voice tests.

use dootdoot_core::{
    ATTACK_TRANSIENT_MIX, ATTACK_TRANSIENT_SECONDS, BODY_LAYER_MIX, UPPER_MID_SPARKLE_MIX,
    attack_transient_sample, body_layer_frequency_hz, body_layer_sample,
    upper_mid_sparkle_frequency_hz, upper_mid_sparkle_sample,
};

#[test]
fn attack_transient_is_deterministic_bounded_and_finite() {
    let elapsed = ATTACK_TRANSIENT_SECONDS * 0.25;
    let first = attack_transient_sample(elapsed, 0.75);
    let second = attack_transient_sample(elapsed, 0.75);

    assert_eq!(first.to_bits(), second.to_bits());
    assert!(first.is_finite());
    assert!(first.abs() <= ATTACK_TRANSIENT_MIX);
    assert_eq!(
        attack_transient_sample(ATTACK_TRANSIENT_SECONDS * 1.25, 0.75).to_bits(),
        0.0_f64.to_bits(),
    );
}

#[test]
fn body_layer_stays_in_bb8_body_band_and_is_bounded() {
    for pitch_hz in [120.0, 440.0, 880.0, 1_760.0] {
        let frequency_hz = body_layer_frequency_hz(pitch_hz);

        assert!(
            (300.0..=700.0).contains(&frequency_hz),
            "body frequency {frequency_hz} escaped the body band",
        );
    }

    let sample = body_layer_sample(0.25, 0.0);

    assert!(sample.is_finite());
    assert!(sample.abs() <= BODY_LAYER_MIX);
    assert!(sample.abs() > 0.0);
}

#[test]
fn upper_mid_sparkle_stays_in_target_band_and_is_bounded() {
    for warble_depth in [-1.0, 0.0, 1.0] {
        for contour in [-1.0, 0.0, 1.0] {
            let frequency_hz = upper_mid_sparkle_frequency_hz(warble_depth, contour);

            assert!(
                (2_000.0..=5_000.0).contains(&frequency_hz),
                "sparkle frequency {frequency_hz} escaped the upper-mid band",
            );
        }
    }

    let sample = upper_mid_sparkle_sample(0.25, ATTACK_TRANSIENT_SECONDS, 1.0);

    assert!(sample.is_finite());
    assert!(sample.abs() <= UPPER_MID_SPARKLE_MIX);
    assert!(sample.abs() > 0.0);
}
