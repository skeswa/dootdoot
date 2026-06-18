//! `VOICE_V11` softened syllable onset.
//!
//! "Hello my name is Sandile" read as percussive and staccato because every
//! syllable snapped to full amplitude within a few milliseconds and every word
//! onset fired a hard transient. The softened attack ramps the envelope over a
//! longer window and spreads a quieter word-onset transient, so syllables bloom
//! instead of click.

use dootdoot_core::{
    ATTACK_TRANSIENT_MIX, ATTACK_TRANSIENT_SECONDS, BASE_SYLLABLE_SECONDS, ENVELOPE_ATTACK_SECONDS,
    amplitude_envelope, attack_transient_sample,
};

#[test]
fn envelope_attack_ramps_gradually_not_as_a_click() {
    // A click-fast onset reaches full amplitude within a few milliseconds. The
    // softened attack is still climbing at 10 ms, so the onset blooms in.
    const {
        assert!(
            ENVELOPE_ATTACK_SECONDS >= 0.012,
            "attack should be long enough to read as a bloom",
        );
    }
    assert!(
        amplitude_envelope(0.010, BASE_SYLLABLE_SECONDS) < 0.6,
        "10 ms into the syllable the onset should still be ramping in",
    );
}

#[test]
fn word_onset_transient_is_gentler_and_longer() {
    // The percussive consonant transient is softer (lower wet mix) and spread
    // over a longer window, so a word onset breathes rather than plucks.
    const {
        assert!(
            ATTACK_TRANSIENT_MIX <= 0.045,
            "word-onset transient should be softer",
        );
        assert!(
            ATTACK_TRANSIENT_SECONDS >= 0.026,
            "word-onset transient should spread over a longer window",
        );
    }

    let peak = (0..=64)
        .map(|step| {
            let elapsed = ATTACK_TRANSIENT_SECONDS * f64::from(step) / 64.0;

            attack_transient_sample(elapsed, 0.0).abs()
        })
        .fold(0.0_f64, f64::max);

    assert!(
        peak <= ATTACK_TRANSIENT_MIX + 1e-9,
        "transient peak {peak} should stay within the softened mix",
    );
}
