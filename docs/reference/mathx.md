# dootdoot `mathx` Contract

`mathx` owns every transcendental used by the audio path. The implementation contract is
`mathx-v1`; changing any table, coefficient, range-reduction rule, or saturation threshold
is sample-affecting and therefore part of `VOICE_V1`.

## API

The runtime API is deliberately small:

- `sin(x)` and `cos(x)` for oscillator phase, vibrato, and ring-modulation.
- `exp(x)` for envelopes and filter coefficient calculations that need exponentials.
- `tanh(x)` implemented through `exp(x)` for the semantic squash.

All functions operate in `f64` and return `f64`. Integer conversion is kept out of this
module; audio conversion uses the project-wide round-half-to-even rule in the WAV/buffer
layer.

## Tables And Polynomials

The pinned v1 shape is:

- Sine/cosine range reduction uses a 4096-interval full-turn table (`2^12`) plus a
  degree-7 local correction polynomial.
- Exponential range reduction uses a 1024-interval table (`2^10`) across one `ln(2)` span
  plus a degree-5 residual polynomial.
- Hyperbolic tangent is computed from the owned exponential and saturates at `|x| >= 20`.

These sizes are intentionally small enough to keep cache behavior predictable and large
enough that the correction polynomials can stay low-degree. T-07 through T-10 will fill in
the coefficients and pinned output fixtures against this contract.

## Determinism Rationale

No libm transcendentals are allowed in the audio path. Platform `sin`, `cos`, `exp`, and
`tanh` implementations can differ by small amounts, and those differences can move final
samples after scaling and round-half-to-even conversion. `mathx` instead uses fixed
tables, fixed polynomial evaluation order, `f64` arithmetic, and no fast-math assumptions.
