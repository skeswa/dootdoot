# VOICE_V11 acceptance — natural voice: softer onset, breathing pace, integrated breath

> Status: **Accepted for VOICE_V11.** The `--version` string reports
> `dootdoot VOICE_V11`. This note records the four tuning slices behind the
> freeze; the binding contract is the regenerated **golden WAV fixtures**
> (`dootdoot-core/tests/fixtures/golden/*.wav`, compared byte-for-byte), not the
> prose here. Acceptance was **by-ear** across a round of feedback.

## Source

Direct listening feedback over several iterations:

1. a plain multi-word phrase ("Hello my name is Sandile") sounded **percussive
   and staccato** — onsets too hard-edged;
2. it wanted **more change of speed throughout**, without leaning on punctuation;
3. a dash ("Hello my name is Sandile - I love cake!!") put a **strange amount of
   noise on the first sentence**; and
4. the **breathiness sounded artifacty** — a separate hiss rather than breath.

## The four slices (each its own concern)

- **T-109 — soften the attack.** Envelope attack ramp 6 ms → 15 ms (quadratic
  ease-in unchanged, so onsets bloom); the per-word onset transient is quieter
  (`ATTACK_TRANSIENT_MIX` 0.07 → 0.04) and longer (`ATTACK_TRANSIENT_SECONDS`
  20 ms → 30 ms) — a breathy consonant, not a pluck.
- **T-110 — breathing pace.** `syllable_rubato_scale(index, total, emphasized)`
  gives each syllable a deterministic duration multiplier: a sinusoidal lilt
  (±8%, ~5.7-syllable period), agogic lengthening on emphasized syllables (+10%),
  and phrase-final lengthening (+10%). Gated on the explicit text path; a
  single-syllable phrase returns exactly `1.0`.
- **T-112 — localize the dash breath.** A dash used to tag its whole preceding
  clause as a breathy `Hesitation` (a 0.45 roughness floor on every syllable).
  Now only the syllable carrying the dash is the hesitation; the preceding clause
  reads as a plain `ChattyReply` statement that trails off (not an inquisitive
  `Probe`). A single-word filler before a dash still reads as a hesitation.
- **T-113 — integrate the breath.** The breath was stationary value-noise (a
  buzzy ~6.3 kHz comb from its fixed 7-sample stride) cross-faded in, so it read
  as a separate hiss — the classic artifact (Klatt; breathy-vowel synthesis). It
  is now pitch-synchronously amplitude-modulated (peaking at the glottal closure
  instant, `breath_closure_modulation`), sourced from a near-white spectrum
  shaped by the formant filter, and mixed additively over the tone.

## Method & research

The breath fix is grounded in formant-synthesis literature: stationary noise is
"perceived as coming from a separate sound source," solved by giving the noise "a
temporal envelope of the same periodicity as the pulse train," with more power
near the glottal closure instant, mixed additively (Klatt synthesizer; "Synthesis
of breathy vowels"; Stanford CCRMA glottal-excitation modeling). By-ear review on
the feedback phrases plus the standing corpus (`scripts/bb8-metrics`) confirmed
the onset reads softer, pace varies within an unpunctuated phrase, the first
sentence before a dash is no longer a wall of breath, and the breath fuses into
the voice — while the droid character is unchanged.

## Determinism & contract

- All changes stay inside the fixed, deterministic, bounded droid parameter space
  (NFR-16); the rubato and the breath modulation are closed-form functions
  (syllable index / glottal phase) — **no randomness** was added.
- The rubato (T-110) is gated on the explicit text path and returns `1.0` for a
  single-syllable phrase, so it does not move the hand-built / empty-chirp /
  neutral-curve path on its own. The attack (T-109) and breath (T-113) are
  *global* synthesis changes that shift every render — which is what the version
  bump is for; no path is byte-identical to `VOICE_V10`.
- Sample **counts** are unchanged on every path (the attack/breath reshape
  amplitude within fixed-duration syllables; the rubato preserves hand-built
  durations). The utterance-estimate-vs-render and double-run determinism tests
  confirm both.
- `ACTIVE_VOICE` is `VOICE_V11`; the golden WAV fixtures were regenerated and the
  double-run determinism test passes.
