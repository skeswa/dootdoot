//! `VOICE_V11` natural aspiration breath: pitch-synchronous amplitude
//! modulation over a whiter, additive noise source.
//!
//! Stationary broadband breath reads as a separate hiss layered over the voice
//! (the "artifacty" quality). The fix, grounded in formant-synthesis literature
//! (Klatt; breathy-vowel synthesis), is to modulate the breath at the glottal
//! rate — louder near the closure instant, quieter mid-period — so it fuses into
//! the voice, and to add it on top of the tone rather than cross-fading it in.

use dootdoot_core::{blend_noise_excitation, breath_closure_modulation};

fn bits(value: f64) -> u64 {
    value.to_bits()
}

#[test]
fn breath_modulation_peaks_at_the_glottal_closure_instant() {
    // The closure instant sits at the period boundary (phase 0); mid-period
    // (phase 0.5) is the quiet trough. A pitch-synchronous breath pulses between
    // the two once per period.
    let closure = breath_closure_modulation(0.0);
    let mid = breath_closure_modulation(0.5);

    assert!(
        closure > mid,
        "breath should be louder at closure ({closure}) than mid-period ({mid})",
    );
    assert!(
        (closure - 1.0).abs() < 1e-9,
        "breath gain should reach full at closure, got {closure}",
    );
}

#[test]
fn breath_modulation_is_periodic_per_glottal_cycle() {
    // The same point in the next period has the same gain — that periodicity is
    // exactly what fuses the noise with the voiced fundamental.
    for phase in [0.0, 0.13, 0.37, 0.6, 0.81] {
        assert!(
            (breath_closure_modulation(phase) - breath_closure_modulation(phase + 1.0)).abs()
                < 1e-9,
            "breath modulation should repeat every glottal period at phase {phase}",
        );
    }
}

#[test]
fn breath_modulation_stays_bounded_and_finite() {
    for step in 0..=128 {
        let phase = f64::from(step) / 64.0; // spans two periods plus odd values
        let gain = breath_closure_modulation(phase);

        assert!(gain.is_finite(), "breath modulation must be finite");
        assert!(
            (0.0..=1.0).contains(&gain),
            "breath modulation must stay in [0, 1], got {gain} at {phase}",
        );
    }

    for phase in [f64::NAN, f64::INFINITY, -2.5] {
        assert!(
            breath_closure_modulation(phase).is_finite(),
            "breath modulation must be finite for degenerate phase {phase}",
        );
    }
}

#[test]
fn breath_blend_is_pitch_synchronous() {
    // At the same sample, breath added at the closure phase must exceed breath
    // added at the mid-period phase (when the underlying noise sample is
    // non-zero) — the audible signature of pitch-synchronous modulation.
    let tonal = 0.3_f64;
    let mut saw_difference = false;

    for index in 0..512 {
        let at_closure = blend_noise_excitation(tonal, index, 0.0, 1.0) - tonal;
        let at_mid = blend_noise_excitation(tonal, index, 0.5, 1.0) - tonal;

        if at_closure.abs() > at_mid.abs() + 1e-9 {
            saw_difference = true;
        }
        // Both are scaled copies of the same noise sample, so they never have
        // opposite signs.
        assert!(
            at_closure * at_mid >= -1e-12,
            "closure and mid-period breath should share the noise sign at {index}",
        );
    }

    assert!(
        saw_difference,
        "breath should be modulated by glottal phase, not stationary",
    );
}

#[test]
fn breath_blend_is_additive_and_keeps_the_tone() {
    // VOICE_V11 mixes breath additively, so the tonal component is preserved
    // (it is not attenuated by a cross-fade toward the noise).
    let tonal = 0.5_f64;
    let mut max_output = 0.0_f64;

    for index in 0..1_024 {
        let blended = blend_noise_excitation(tonal, index, 0.2, 1.0);

        assert!(blended.is_finite());
        max_output = max_output.max(blended.abs());
    }

    // Additive breath can push the instantaneous level above the bare tone.
    assert!(
        max_output > tonal,
        "additive breath should ride on top of the tone, got peak {max_output}",
    );
}

#[test]
fn breath_blend_amount_zero_keeps_clean_periodicity() {
    for index in 0..64 {
        let tonal = 0.4 * f64::from(index % 7);

        assert_eq!(
            bits(blend_noise_excitation(tonal, index, 0.3, 0.0)),
            bits(tonal),
            "ordinary syllables must stay cleanly periodic",
        );
    }
}
