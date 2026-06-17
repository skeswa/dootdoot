# VOICE_V9 audible-punctuation acceptance

Status: **Accepted for VOICE_V9**.

`VOICE_V9` makes the five marks a writer reaches for — **question, exclamation, period,
dash, and ellipsis** — each render as a distinct, recognisable prosodic gesture rather than
colliding or being dropped. It keeps the four PCA semantic axes as the learnable core and
reuses every `VOICE_V2`–`VOICE_V8` performance channel; it only reshapes the per-mark
boundary signature.

## Scope

- `dootdoot --version` reports `dootdoot VOICE_V9`.
- The embedded semantic mapping in `assets/dootdoot_asset_v1.doot` is unchanged; the PCA
  mapping is unchanged.
- **R2 — normalisation.** A run of two or more ASCII periods (`...`, `....`) collapses to a
  single trailing-off **ellipsis** hesitation marker instead of a stutter of falling-glide
  periods, and a stacked terminal run (`?!`, `!!!`) collapses to its first contour. The
  engine already first-wins on consecutive punctuation, so the only sample-affecting case is
  `...` becoming an ellipsis.
- **R4 — closure vs continuation.** Clause marks (`,` `;` `:`) carry a shallow
  **continuation rise** (a new `+1.5` st glide) and drop their final lowering, so a comma
  reads as "more coming" against a period's close.
- **R1 — settle vs punch.** A period falls deep to a quiet **settle** (`−1.40` st) while an
  exclamation falls only shallowly (`−0.60` st) from its raised, emphasized **punch** peak;
  a question keeps its suppressed rising close. The two falling marks were previously
  acoustically identical apart from terminal role.
- **R3 — dash vs ellipsis tail shape.** A hesitation marker now shapes the preceding
  syllable's **tail**: a dash clips it to silence with a steep gate (abrupt cutoff), an
  ellipsis decays it exponentially (trailing off). The default sustained tail is a
  transparent unity gain, so every non-hesitation syllable is byte-identical.
- **R5 — question rise.** The question gains a dedicated, wider terminal rise (`4.5` st,
  vs the `3.0` st generic glide) with a small pre-final dip (`L*`) so the lift gathers then
  rises (`H-H%`) even on a short final word; declination stays suppressed across the whole
  final segment.
- The committed golden WAV hashes were regenerated under `VOICE_V9`, including new `dash`
  and `ellipsis` corpus fixtures (the ellipsis fixture also locks the R2 ASCII `...`
  normalisation).

## Directional Check

Derived from
[`punctuation-prosody-audibility.md`](../research/punctuation-prosody-audibility.md). Each
mark is rendered on a minimal carrier (`now.` / `now!` / `now?`, and `wait, go` /
`wait - go` / `wait ... go`) and decoded mono 44.1 kHz through the locked uv harness
(`scripts/acoustic_metrics.py`). The point is **separation**: each mark occupies a distinct
acoustic cell.

| Measurement              | `now.` | `now!` | `now?` | `…, go` | `… - go` | `… ... go` | Read                                                            |
| ------------------------ | -----: | -----: | -----: | ------: | -------: | ---------: | --------------------------------------------------------------- |
| Zero-crossing rate       |   1770 |   1220 |   1130 |    1630 |     1400 |       1540 | the period settle stays buzzier; the flourishes calm.           |
| Spectral flux median     |  0.444 |  0.334 |  0.333 |   0.213 |    0.232 |      0.316 | the period is the most animated terminal close.                 |
| Dominant peak range (Hz) |    883 |    711 |    732 |     366 |      237 |        883 | period / exclamation / question each sweep a different span.    |
| Harmonicity median       |  0.731 |  0.833 |  0.823 |   0.818 |    0.731 |      0.679 | the ellipsis tail is the roughest as it decays.                 |
| Active fraction          |   0.37 |  0.348 |  0.348 |   0.577 |    0.229 |      0.209 | the comma is the most connected ("more coming").                |
| Max internal gap (ms)    |      0 |      0 |      0 |     163 |      964 |        964 | dash and ellipsis share the role-gated turn gap (see below).    |
| Median active island ms  |    197 |    186 |    186 |     174 |      157 |        139 | the ellipsis trails off into the shortest island.               |
| Tremolo depth            |  0.174 |  0.165 |  0.162 |   0.196 |    0.204 |      0.277 | the decayed ellipsis tail reads as the deepest trailing warble. |

Two reads matter most:

- **The three terminal marks separate.** `now.` (deep settle, chatty role) is the buzziest
  and most animated; `now!` and `now?` calm into the flourish, and split from each other on
  pitch direction (fall vs rise) and dominant-peak span. The previously-identical period and
  exclamation are now distinct.
- **Dash and ellipsis separate on _tail shape_, not the gap.** Both inherit the same
  role-gated **964 ms** turn gap, so the marker's own 340/500 ms rest is masked — exactly as
  the research note predicted. The distinguishing cues are therefore the preceding
  syllable's tail: the clipped dash holds harmonicity at 0.731 and a 157 ms island, while
  the decayed ellipsis trails to 0.679 harmonicity, a 139 ms island, and a deeper 0.277
  tremolo. The tail-shape directive, not the rest length, is doing the work.

## Acceptance

`VOICE_V9` is accepted as the active audible-punctuation contract. The directional metrics
above are a tuning aid, not the sample-level contract; the **golden WAV hashes** remain the
byte-exact contract and were regenerated under `VOICE_V9`. Determinism, boundedness, and the
no-runtime-randomness and no-PCA-change invariants are preserved.
