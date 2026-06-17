//! `VOICE_V7` deterministic noise/breath excitation primitive tests.

use dootdoot_core::{
    NOISE_BREATH_MAX_MIX, PhraseRole, blend_noise_excitation, noise_breath_sample,
    syllable_roughness_amount,
};

fn bits(value: f64) -> u64 {
    value.to_bits()
}

#[test]
fn agitated_accent_bursts_into_the_noisy_band() {
    // VOICE_V10: a body accent in a high-arousal, negative-valence utterance
    // breaks into a rough/noisy burst so a single gesture leaves the tonal band.
    let agitated = syllable_roughness_amount(PhraseRole::ChattyReply, 0.85, 0.5, -1.0, 1.0);

    assert!(
        agitated >= 0.7,
        "an agitated accent should burst rough, got {agitated}",
    );
}

#[test]
fn calm_or_positive_accents_keep_the_base_roughness() {
    // Positive valence (happy) or low arousal must not trigger the burst; the
    // accent stays in its V8 body range.
    let happy = syllable_roughness_amount(PhraseRole::ChattyReply, 0.85, 0.5, 1.0, 1.0);
    let calm = syllable_roughness_amount(PhraseRole::ChattyReply, 0.85, 0.5, -1.0, 0.2);

    assert!(
        happy <= 0.4,
        "a happy accent should stay in the body range, got {happy}"
    );
    assert!(
        calm <= 0.4,
        "a low-arousal accent should stay in the body range, got {calm}"
    );
}

#[test]
fn non_accent_and_neutral_syllables_never_burst() {
    // A non-accent body syllable (below the whistle gate) and a hand-built /
    // neutral syllable stay clean even in an agitated utterance.
    let non_accent = syllable_roughness_amount(PhraseRole::ChattyReply, 0.40, 0.5, -1.0, 1.0);
    let neutral = syllable_roughness_amount(PhraseRole::ChattyReply, 0.0, 0.0, -1.0, 1.0);

    assert!(
        non_accent <= 0.4,
        "non-accent body should not burst, got {non_accent}"
    );
    assert!(
        neutral.abs() < 1e-12,
        "neutral hand-built syllable must stay clean, got {neutral}"
    );
}

#[test]
fn noise_breath_mix_is_bounded() {
    const {
        assert!(NOISE_BREATH_MAX_MIX > 0.0 && NOISE_BREATH_MAX_MIX <= 0.75);
    }
}

#[test]
fn noise_breath_amount_zero_is_silent() {
    for index in 0..32 {
        assert_eq!(bits(noise_breath_sample(index, 0.0)), bits(0.0));
    }
}

#[test]
fn noise_breath_blend_amount_zero_keeps_clean_periodicity() {
    for index in 0..64 {
        let tonal = 0.4 * f64::from(index % 7);

        assert_eq!(
            bits(blend_noise_excitation(tonal, index, 0.0)),
            bits(tonal),
            "ordinary syllables must stay cleanly periodic",
        );
    }
}

#[test]
fn noise_breath_is_deterministic() {
    for index in [0, 1, 7, 41, 1_000, 44_099] {
        assert_eq!(
            bits(noise_breath_sample(index, 0.8)),
            bits(noise_breath_sample(index, 0.8)),
        );
    }
}

#[test]
fn noise_breath_is_bounded_and_finite_over_a_grid() {
    let amounts = [-1.0, 0.0, 0.25, 0.7, 1.0, 2.0, f64::NAN, f64::INFINITY];

    for amount in amounts {
        for index in 0..4_096 {
            let value = noise_breath_sample(index, amount);

            assert!(value.is_finite(), "noise breath must be finite");
            assert!(
                value.abs() <= 1.0,
                "noise breath must stay bounded: {value} at {index}/{amount}",
            );
        }
    }
}

fn usize_to_f64(value: usize) -> f64 {
    u32::try_from(value).map(f64::from).expect("count fits u32")
}

#[test]
fn noise_breath_is_aperiodic_and_active() {
    let samples = (0_u32..2_048)
        .map(|index| noise_breath_sample(index, 1.0))
        .collect::<Vec<_>>();
    let mean = samples.iter().sum::<f64>() / usize_to_f64(samples.len());
    let variance = samples
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / usize_to_f64(samples.len());

    assert!(
        variance > 0.01,
        "noise breath should carry real aperiodic energy: variance {variance}",
    );

    // Not a single periodic tone: it should not repeat at the oscillator scale.
    let short_period_matches = (0_usize..1_024)
        .filter(|&index| (samples[index] - samples[index + 64]).abs() < 1e-9)
        .count();

    assert!(
        short_period_matches < 64,
        "noise breath should not be a short periodic tone: {short_period_matches} matches",
    );
}

#[test]
fn noise_breath_blend_roughens_within_bounds() {
    let amount = 1.0;
    let tonal = 0.5_f64;
    let mix = amount * NOISE_BREATH_MAX_MIX;
    // |tonal*(1-mix) + noise*mix| <= |tonal| + mix, since |noise| <= 1.
    let output_bound = tonal.abs() + mix;
    let mut max_delta = 0.0_f64;

    for index in 0..1_024 {
        let blended = blend_noise_excitation(tonal, index, amount);

        assert!(blended.is_finite());
        assert!(
            blended.abs() <= output_bound + 1e-9,
            "blend output must stay bounded: {blended} > {output_bound}",
        );
        max_delta = max_delta.max((blended - tonal).abs());
    }

    assert!(
        max_delta > 0.0,
        "blend should audibly roughen the tonal source: max delta {max_delta}",
    );
}
