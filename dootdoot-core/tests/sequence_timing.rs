//! `VOICE_V7` role-gated timing primitive tests.

use dootdoot_core::{
    KnobSet, ROLE_LONG_PAUSE_MAX_SAMPLES, ROLE_LONG_PAUSE_MIN_SAMPLES,
    STAGED_REPLY_REST_MAX_SAMPLES, STAGED_REPLY_REST_MIN_SAMPLES, SequenceEvent, SquashedVector,
    SyllableTiming, assemble_knobs, estimate_utterance_sample_count, render_canonical_buffer,
    role_long_pause_samples, staged_reply_rest_samples,
};

fn neutral_knobs() -> KnobSet {
    let neutral = SquashedVector::new([0.0, 0.0, 0.0, 0.0]);

    assemble_knobs(neutral, neutral)
}

fn max_zero_run(samples: &[i16]) -> usize {
    let mut best = 0;
    let mut run = 0;

    for &sample in samples {
        if sample == 0 {
            run += 1;
            best = best.max(run);
        } else {
            run = 0;
        }
    }

    best
}

#[test]
fn timing_constants_cover_turn_and_reply_ranges() {
    const {
        assert!(ROLE_LONG_PAUSE_MIN_SAMPLES >= 26_000 && ROLE_LONG_PAUSE_MIN_SAMPLES <= 27_000);
        assert!(ROLE_LONG_PAUSE_MAX_SAMPLES >= 52_000 && ROLE_LONG_PAUSE_MAX_SAMPLES <= 53_500);
        assert!(STAGED_REPLY_REST_MIN_SAMPLES >= 1_200 && STAGED_REPLY_REST_MIN_SAMPLES <= 1_500);
        assert!(STAGED_REPLY_REST_MAX_SAMPLES >= 3_400 && STAGED_REPLY_REST_MAX_SAMPLES <= 3_700);
    }
}

#[test]
fn role_long_pause_maps_amount_to_bounded_range() {
    assert_eq!(role_long_pause_samples(0.0), ROLE_LONG_PAUSE_MIN_SAMPLES);
    assert_eq!(role_long_pause_samples(1.0), ROLE_LONG_PAUSE_MAX_SAMPLES);
    assert_eq!(role_long_pause_samples(-2.0), ROLE_LONG_PAUSE_MIN_SAMPLES);
    assert_eq!(role_long_pause_samples(4.0), ROLE_LONG_PAUSE_MAX_SAMPLES);
    assert_eq!(
        role_long_pause_samples(f64::NAN),
        ROLE_LONG_PAUSE_MIN_SAMPLES
    );

    let mid = role_long_pause_samples(0.5);

    assert!(mid > ROLE_LONG_PAUSE_MIN_SAMPLES && mid < ROLE_LONG_PAUSE_MAX_SAMPLES);
}

#[test]
fn staged_reply_rest_maps_amount_to_bounded_range() {
    assert_eq!(
        staged_reply_rest_samples(0.0),
        STAGED_REPLY_REST_MIN_SAMPLES
    );
    assert_eq!(
        staged_reply_rest_samples(1.0),
        STAGED_REPLY_REST_MAX_SAMPLES
    );
    assert_eq!(
        staged_reply_rest_samples(9.0),
        STAGED_REPLY_REST_MAX_SAMPLES
    );
    assert_eq!(
        staged_reply_rest_samples(f64::NAN),
        STAGED_REPLY_REST_MIN_SAMPLES
    );
}

#[test]
fn default_timing_keeps_word_boundary_bridged() {
    let knobs = neutral_knobs();
    let events = [
        SequenceEvent::syllable(knobs, false),
        SequenceEvent::syllable(knobs, false),
    ];
    let buffer = render_canonical_buffer(&events);

    // A bridged word boundary fills the gap with tone, so there is no long rest.
    assert!(
        max_zero_run(&buffer) < ROLE_LONG_PAUSE_MIN_SAMPLES as usize,
        "default timing should bridge the word gap with tone",
    );
}

#[test]
fn suppressed_bridge_inserts_a_real_rest() {
    let knobs = neutral_knobs();
    let long_pause = role_long_pause_samples(0.5);
    let timing = SyllableTiming::default()
        .with_pause_override(long_pause)
        .suppress_bridge();
    let events = [
        SequenceEvent::syllable_with_timing(knobs, false, timing),
        SequenceEvent::syllable(knobs, false),
    ];
    let buffer = render_canonical_buffer(&events);

    assert!(
        max_zero_run(&buffer) >= ROLE_LONG_PAUSE_MIN_SAMPLES as usize,
        "a suppressed bridge with a long pause should leave a real silent rest",
    );
}

#[test]
fn timing_directives_keep_estimate_and_render_in_sync() {
    let knobs = neutral_knobs();
    let timing = SyllableTiming::default()
        .with_pause_override(role_long_pause_samples(1.0))
        .suppress_bridge();
    let events = [
        SequenceEvent::syllable_with_timing(knobs, false, timing),
        SequenceEvent::syllable(knobs, false),
    ];
    let estimate = estimate_utterance_sample_count(&events);

    assert_eq!(
        usize::try_from(estimate).expect("estimate fits usize"),
        render_canonical_buffer(&events).len(),
        "timing override must keep estimate aligned with render",
    );
}

#[test]
fn long_pause_override_extends_total_duration() {
    let knobs = neutral_knobs();
    let baseline = [
        SequenceEvent::syllable(knobs, false),
        SequenceEvent::syllable(knobs, false),
    ];
    let timing = SyllableTiming::default()
        .with_pause_override(role_long_pause_samples(1.0))
        .suppress_bridge();
    let staged = [
        SequenceEvent::syllable_with_timing(knobs, false, timing),
        SequenceEvent::syllable(knobs, false),
    ];

    assert!(estimate_utterance_sample_count(&staged) > estimate_utterance_sample_count(&baseline));
}
