# Making Punctuation Audible: A Boundary-Tone Plan for Sound Planning

> Status: **research / directional**. The user's goal: it should be **clearly audible**
> when the text contains a **question**, **exclamation**, **explicit period**, **dash**, or
> **ellipsis** — each recognisably distinct from the others, not just from neutral text.
>
> This note (1) surveys how modern TTS systems make these five marks audible, (2) measures
> exactly what `VOICE_V8` does today and where the five marks collide or get dropped, and
> (3) proposes concrete sound-planning changes mapped onto dootdoot's existing primitives.
>
> It does **not** change the voice contract. Every recommendation here is sample-affecting
> and would require a new voice version (`V9`) plus regenerated golden WAV hashes
> (CLAUDE.md "load-bearing invariants").

## Summary

dootdoot already has the right machinery — per-mark final glides, boundary-strength pauses,
declination/lowering, and a separate quiet-rest path for dash/ellipsis. But four of the five
marks the user cares about are **not reliably distinguishable today**:

1. **Period and exclamation share the same local boundary signature.** Both map to a
   `Falling` final glide and the same 240 ms long pause (`sequence.rs:360`, `synth.rs:83`).
   They are not byte-identical end-to-end — `!` raises the utterance arousal proxy, and a
   final `!` gets the `TerminalFlourish` role — but the punctuation control itself still has
   no mark-specific **punch**. A period needs an audible **settle**; an exclamation needs an
   audible **punch**, especially away from the final syllable.
2. **`...` typed as three ASCII periods routes through period controls, not ellipsis.** Only
   the single Unicode `…` (U+2026) is recognised as an ellipsis (`sequence.rs:127`). The far
   more common `...` tokenises into three separate `Period` markers in `--explain`. The audio
   path does _not_ literally render three falling glides or three additive 240 ms pauses
   because consecutive punctuation uses the first marker's glide and the longest single pause;
   still, the intended "half-finished thought" gesture almost never fires in practice.
3. **Dash and ellipsis differ only by the length of a silent gap** (340 ms vs 500 ms;
   `sequence.rs:105,108`). Both are "quiet rest + `Hesitation` role." Linguistically these
   are opposites — a dash is an _abrupt cutoff_, an ellipsis is a _gradual trailing-off_ —
   and that contrast lives in the **shape of the preceding syllable's tail**, which today is
   identical for both.
4. **The explicit period is weakly terminal.** It has no dedicated role (it falls to the
   `_ => ChattyReply` arm of `segment_role`, `performance.rs:414`), and the clause marks
   (`,` `;` `:`) all collapse to one neutral-glide `Aside` bucket. There is no
   _continuation-vs-closure_ contrast to make a period read as "done."

The fix is not new primitives — it is **assigning each of the five marks a distinct
boundary-tone signature** across the knobs we already have (final glide direction _and
magnitude_, final lengthening, pause, brightness/intensity, and tail shape), plus a small
normalisation pass so `...` becomes an ellipsis.

There is also a source-of-truth mismatch to resolve before implementation: `spec.md` FR-82,
`docs/plan.md` T-84, and `design.md` §8.8 say ASCII `...` should already be a `VOICE_V7`
hesitation marker, but the current code and `design.md` §3.3 only treat the single Unicode
ellipsis (`…`) that way. This note describes current `VOICE_V8` behavior as observed from
code and `--explain`; the V9 work should explicitly reconcile those documents.

## 1. How modern TTS makes these marks audible

Modern TTS — from ToBI-driven concatenative engines to SSML front-ends on neural
voices — encodes terminal punctuation as a small set of **boundary tones** plus
**phrase-final timing and intensity** adjustments. The canonical correlates:

| Mark               | Boundary tone (ToBI)                    | Pitch                                                      | Timing                                     | Intensity                      | Percept                   |
| ------------------ | --------------------------------------- | ---------------------------------------------------------- | ------------------------------------------ | ------------------------------ | ------------------------- |
| Statement `.`      | `L-L%` (low fall to floor)              | final **fall to bottom of range**, full declination        | strong **final lengthening**, then settle  | drops off                      | closure / "done"          |
| Question `?`       | `L* H-H%` / `H-H%` (high rise)          | final **rise**, declination suppressed                     | mild lengthening                           | sustained                      | open / "your turn"        |
| Exclamation `!`    | `H* L-L%` from a **raised** peak        | **expanded pitch range**, higher peak, steeper move        | sharper attack, can be faster              | **raised loudness / emphasis** | emphatic punch            |
| Clause `,` `;` `:` | `L-H%` / `H-` (continuation rise)       | slight **rise or level**, not to floor                     | short pause, light lengthening             | sustained                      | "more coming"             |
| Dash `—`           | truncation / suspension                 | contour **clipped mid-move**, often a held or glottal stop | **abrupt** cutoff, then pause              | cut short                      | interruption / break      |
| Ellipsis `…`       | half-cadent / level (`L-` or sustained) | **level or shallow fall**, held                            | **lengthen + amplitude decay**, long pause | fades out                      | trailing off / unfinished |

Two themes matter for dootdoot:

- **Direction _and_ degree.** A period and an exclamation can _both_ end in a fall, but the
  exclamation falls from a **raised, widened** register with more loudness, and the period
  falls **all the way to the declination floor** with final lengthening. The contrast is in
  the magnitude and the surrounding range, not just the sign of the glide. SSML exposes this
  as separate `pitch`, `range`, `rate`, and `volume`/`emphasis` levers
  ([SSML](https://en.wikipedia.org/wiki/Speech_Synthesis_Markup_Language),
  [milvus: prosody control](https://milvus.io/ai-quick-reference/how-is-prosody-controlled-in-modern-tts-systems)).
- **Dash ≠ ellipsis is a _shape_ contrast.** Both are pauses, but the dash is an _abrupt_
  break (truncated phonation, a suspended/held tone) while the ellipsis _trails off_
  (sustained level or "semi-cadent" pitch with amplitude decay and final lengthening — the
  classic "half-finished thought"). The two marks were historically the same glyph precisely
  because both mark a break; speech keeps them distinct through the **tail of the preceding
  syllable**, not the gap alone
  ([Ellipsis](https://en.wikipedia.org/wiki/Ellipsis),
  [Boundary tone](https://en.wikipedia.org/wiki/Boundary_tone),
  [ToBI](https://en.wikipedia.org/wiki/ToBI)).

## 2. What `VOICE_V8` does today

### 2.1 The two punctuation paths

dootdoot splits punctuation across two enums:

- **`ProsodicPunctuation`** (`sequence.rs:203`) — `?` `.` `!` `,` `;` `:`. Each contributes a
  **final glide** (`sequence.rs:360`) and a **boundary pause** (`sequence.rs:353`), and feeds
  `segment_role` (`performance.rs:398`) + the phrase-final prosody in `phrase.rs`
  (`final_lowering_semitones`, `pitch_reset_semitones`, `pre_boundary_lengthening`).
- **`HesitationMarker`** (`sequence.rs:115`) — `-` `--` `—` `–` and `…`. These are _not_
  voiced; they impose a **quiet, bridge-suppressed rest** on the _preceding_ syllable
  (`sequence.rs:142`) and push that segment into the `Hesitation` role.

### 2.2 Current per-mark signature

| Mark       | Parsed as             | Final glide       | Pause            | Terminal role                     | Final lowering      |
| ---------- | --------------------- | ----------------- | ---------------- | --------------------------------- | ------------------- |
| `?`        | Question              | **Rising**        | 240 ms           | `TerminalFlourish` (yelp/whistle) | suppressed (0)      |
| `!`        | Exclamation           | **Falling**       | 240 ms           | `TerminalFlourish`                | sentence (−0.90 st) |
| `.`        | Period                | **Falling**       | 240 ms           | _default_ (`ChattyReply`)         | sentence (−0.90 st) |
| `,`        | Comma                 | Neutral           | 150 ms           | `Aside`                           | clause (−0.20 st)   |
| `;`        | Semicolon             | Neutral           | 150 ms           | `Aside`                           | clause (−0.20 st)   |
| `:`        | Colon                 | Neutral           | 150 ms           | `Aside`                           | clause (−0.20 st)   |
| `- — – --` | Dash (hesitation)     | — (rest on prior) | **340 ms quiet** | `Hesitation`                      | —                   |
| `…`        | Ellipsis (hesitation) | — (rest on prior) | **500 ms quiet** | `Hesitation`                      | —                   |

Verified with `--explain` on `"wait... no, stop - now"`: `...` appears as **three**
`control:period · falling glide · pause 240 ms` rows, while `-` appears as one
`hesitation-dash · quiet rest 340 ms` row. In rendered audio, those three period rows collapse
through the existing "first marker shapes, longest single pause wins" rule rather than
creating three additive pauses; the remaining problem is that the boundary is still
period-shaped, not ellipsis-shaped.

### 2.3 The collisions

- `.` vs `!`: same local glide + pause. `!` can differ through utterance arousal and final
  `TerminalFlourish`, but it has no distinct exclamation boundary shape. Question already
  reads (it's the only riser, and it's the only mark that keeps pitch up).
- `...` ⇒ period controls: the common ASCII ellipsis is explain-visible as three period rows
  and audio-visible as a period-shaped boundary, not one trailing-off gesture.
- `-` vs `…`: same quiet-rest mechanism, only the gap length differs.
- `.` mid-utterance: no role of its own; clause marks share one bucket — closure and
  continuation are not contrasted.

## 3. Recommendations for sound planning (`VOICE_V9`)

The organising idea: give each mark a distinct point in a **(final pitch movement) ×
(intensity / finality) × (tail shape)** space. Most of this can reuse existing planner and
synth controls (role curves, final lowering, duration, pause overrides, brightness pressure,
whistle/tension), but the work is not purely constant tuning: V9 needs a small per-marker
boundary-signature model for glide magnitude/shape, plus a tail-shape directive for
dash/ellipsis. Each recommendation below is independently implementable and testable inside
a V9 branch; if any slice ships separately, it needs its own voice version and golden
regeneration.

### R1 — Split period from exclamation: _settle_ vs _punch_

Today both fall. Differentiate by **register, magnitude, intensity, and timing**, mirroring
the `L-L%` (statement) vs raised-peak `H* L-L%` (exclamation) split:

- **Period = definitive settle.** Deepen the final fall to the declination floor, increase
  `pre_boundary_lengthening` (slow the last syllable), and _lower_ brightness/intensity on
  the terminal syllable. Give it a dedicated quiet, closing role (or a "settle" curve set)
  rather than falling through to `ChattyReply`. Suppress the whistle/flourish — a period is a
  statement, not a flourish.
- **Exclamation = emphatic punch.** Raise the terminal pitch _peak_ and **widen the range**
  before the fall, steepen `pitch_velocity`, add a **brightness/sparkle burst**
  (`brightness_pressure` ↑, the event-sparkle path in `synth.rs`), and keep
  `TerminalFlourish` but flavour it as a punch (fall from a raised peak) rather than the
  question's rise. Avoid promising literal envelope-attack shortening unless V9 adds a
  bounded attack-shape control; today the attack envelope/transient durations are fixed voice
  constants. The existing `archetype_tension` lever already drives the flourish whistle —
  bias exclamation toward a high-energy fall, question toward the rise.

Acceptance feel: `"now."` vs `"now!"` vs `"now?"` must be three obviously different endings.

### R2 — Normalise `...` (and friends) into one ellipsis

Before tokenisation, or in a deterministic token-normalisation pass immediately after it,
collapse runs of ASCII periods (`...`, `. . .`, `....`) and the spaced/Unicode variants into
a single `Ellipsis` `HesitationMarker`, and collapse `--`/`---`/spaced hyphens into one
`Dash`. `--` and the Unicode dashes are already handled (`sequence.rs:126`); the gap is
specifically `...` → ellipsis and multi-dot runs. This is the single highest-leverage change:
it makes the ellipsis gesture actually reachable for the way people type, and it stops the
period-control stutter. (Decide the boundary deliberately: `!?` / `?!` interrobang runs
should pick one terminal contour, not stack two.)

First reconcile the existing source-of-truth mismatch: either narrow FR-82/T-84 to the
current Unicode-only implementation, or make ASCII `...` part of the V9 audibility contract
and update the historical V7 text to say it was planned but not implemented.

### R3 — Make dash and ellipsis _shape_ the tail differently, not just the gap

Keep both as quiet rests, but contrast the **preceding syllable's tail** so the break type is
audible before the silence:

- **Dash = abrupt cutoff.** Truncate/clip the prior syllable's tail (shorten its release,
  hard amplitude gate), freeze the contour mid-move (a brief _held_ suspension rather than a
  resolved glide), then a dash rest. Percept: interruption.
- **Ellipsis = trailing off.** _Lengthen_ the prior syllable and apply a gentle amplitude
  **decay**, hold a level or shallow-falling (half-cadent) pitch — not a resolved fall to the
  floor — round off brightness, then a longer ellipsis rest. Percept: unfinished thought.

This recruits the `SyllableTiming` path the hesitation markers already own
(`with_pause_override`, `suppress_bridge`, `mark_hesitation`, `sequence.rs:142`); the new
pieces are a **tail-shape directive** (clip vs decay) and a **held-vs-level contour** on the
hesitating syllable. V9 also needs to decide pause precedence: current V8 deployment can
replace the marker's 340/500 ms pause with the role-gated ~930 ms hesitation turn gap before
the next phrase, so the marker-specific rest length is not reliably audible in common
`"wait - no"` / `"wait … no"` shapes unless the marker pause wins or the long-pause planner
becomes marker-aware. The 340/500 ms gap difference should stay only as a secondary cue.

### R4 — Contrast closure against continuation

Give the explicit period a real terminal identity and let clause marks read as "more coming":

- **Period**: low fall to floor + final lengthening + the R1 settle role.
- **Clause `,` `;` `:`**: adopt a shallow **continuation rise or level** (`L-H%`) instead of
  the flat neutral glide, with the existing shorter pause. This makes a period audibly
  _closed_ against a comma's _open_, the same way the question/period rise/fall pair already
  contrasts. This requires a new per-marker glide shape or a `Continuation` final-glide
  variant; today's `SyllableFinalGlide` only has `Neutral`, `Rising`, and `Falling`, with one
  fixed 3-semitone magnitude. (Optionally split `:` toward "presenting/listing" later — out
  of scope here.)

### R5 — Strengthen the question rise (after glide-shape support)

The question already reads, but widen the terminal rise and add a small pre-final dip (`L*`
before `H-H%`) so the rise is unmistakable on short final words, and confirm declination stays
suppressed through the whole final segment (not just the last syllable). The confirmation is
cheap; the wider rise and pre-final dip depend on the same per-marker glide-shape support as
R4 because today's question glide is only the fixed 3-semitone `Rising` variant.

### Proposed target signature (after R1–R5)

| Mark        | Final movement                       | Intensity / finality         | Tail shape     | Gap                           |
| ----------- | ------------------------------------ | ---------------------------- | -------------- | ----------------------------- |
| `?`         | strong **rise**, declination off     | sustained                    | resolved up    | long                          |
| `!`         | fall from a **raised, widened** peak | **loud / sparkle punch**     | sharp          | long                          |
| `.`         | fall **to the floor**                | quiet **settle**, lengthened | resolved down  | long                          |
| `,` `;` `:` | shallow **rise / level**             | sustained, light             | resolved level | medium                        |
| `—`         | contour **clipped mid-move**         | **held / abrupt**            | truncated      | dash rest; long-gap gated     |
| `…`         | **level / shallow** hold             | fades, lengthened            | **decayed**    | ellipsis rest; long-gap gated |

## 4. Contract & sequencing notes

- **All of R1–R5 are sample-affecting.** If they ship as one batch, they require one
  `VOICE_V9` bump, regenerated golden WAV hashes (`DOOTDOOT_REGEN_GOLDEN=1 …`), and a
  `docs/validation/voice-v9-*.md` acceptance note. If any recommendation ships before or
  after that batch, it needs its own voice version and golden fixture set. Do **not** ship
  any of them under `V8`.
- **Order of work, lowest-risk first:** R2 (normalisation — pure front-end, big perceptual
  win) → R4 (closure/continuation glides) → R1 (period/exclamation split) → R3 (dash/ellipsis
  tail shapes — needs a new tail directive) → R5 (question polish). R2 can land as its own
  task because it changes _which markers fire_, which every later recommendation depends on.
- **TDD hooks:** `--explain` already prints the per-token glide/pause/role and the curve grid,
  so each mark's signature is snapshot-testable at the planner level before any golden-WAV
  assertion. Add `performance_planner` / `phrase_smoothing` tests asserting the distinct
  glide + role + tail per mark, then pin bytes with goldens.
- **Spec/plan sync:** first fix the existing ASCII-ellipsis contradiction across FR-82,
  T-84, `design.md` §8.8, `design.md` §3.3, and code. Then add `FR-*` entries for the
  five-mark audibility contract and a Phase task block (`T-NN`) per CLAUDE.md's
  source-of-truth requirement.
- **Acoustic validation:** use `scripts/acoustics` to confirm `"now."` / `"now!"` / `"now?"`
  separate on dominant-peak range, 2–5 kHz share (exclamation punch), and final-pitch
  direction, and that `"wait — go"` vs `"wait … go"` separate on tail-energy decay and
  pre-gap duration.

## Sources

- [ToBI (Tones and Break Indices)](https://en.wikipedia.org/wiki/ToBI)
- [Boundary tone](https://en.wikipedia.org/wiki/Boundary_tone)
- [Prosody (linguistics)](<https://en.wikipedia.org/wiki/Prosody_(linguistics)>)
- [Ellipsis](https://en.wikipedia.org/wiki/Ellipsis)
- [Exclamation mark](https://en.wikipedia.org/wiki/Exclamation_mark)
- [Speech Synthesis Markup Language (SSML)](https://en.wikipedia.org/wiki/Speech_Synthesis_Markup_Language)
- [How is prosody controlled in modern TTS systems? (Milvus)](https://milvus.io/ai-quick-reference/how-is-prosody-controlled-in-modern-tts-systems)
