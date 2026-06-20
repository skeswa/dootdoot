//! `VOICE_V11` organic intra-phrase rubato.
//!
//! Without punctuation, every syllable used to pace identically, so a phrase
//! read as metronomic. The rubato scale gives each syllable a small,
//! deterministic duration variation from its position — a lilt that quickens
//! and broadens across the phrase, agogic lengthening on emphasized syllables,
//! and natural phrase-final broadening — so speed breathes without needing
//! commas.

use dootdoot_core::syllable_rubato_scale;

#[test]
fn single_syllable_phrase_has_no_internal_rubato() {
    assert_eq!(
        syllable_rubato_scale(0, 1, false).to_bits(),
        1.0_f64.to_bits()
    );
    assert_eq!(
        syllable_rubato_scale(0, 0, false).to_bits(),
        1.0_f64.to_bits()
    );
}

#[test]
fn phrase_quickens_and_broadens_without_punctuation() {
    let total = 7;
    let scales: Vec<f64> = (0..total)
        .map(|index| syllable_rubato_scale(index, total, false))
        .collect();

    assert!(
        scales[..total - 1].iter().any(|&scale| scale < 1.0),
        "some interior syllable should quicken below the nominal pace: {scales:?}",
    );
    assert!(
        scales.iter().any(|&scale| scale > 1.0),
        "some syllable should broaden above the nominal pace: {scales:?}",
    );
    assert!(
        *scales.last().expect("non-empty") > 1.0,
        "the final syllable should broaden (phrase-final lengthening): {scales:?}",
    );
}

#[test]
fn emphasis_lengthens_a_syllable_agogically() {
    let plain = syllable_rubato_scale(2, 6, false);
    let stressed = syllable_rubato_scale(2, 6, true);

    assert!(
        stressed > plain,
        "an emphasized syllable should lengthen: {stressed} <= {plain}",
    );
}

#[test]
fn rubato_stays_bounded_and_finite() {
    for total in [0_usize, 1, 2, 5, 12, 40] {
        for index in 0..total.max(1) {
            for emphasized in [false, true] {
                let scale = syllable_rubato_scale(index, total, emphasized);

                assert!(
                    scale.is_finite() && (0.8..=1.3).contains(&scale),
                    "scale {scale} out of range at {index}/{total}",
                );
            }
        }
    }
}
