# Squash Function

`FORMAT_V1` uses a tanh z-score squash for projected PCA values.

For each retained PCA axis, `xtask` projects every token embedding, then records the
population mean and population standard deviation of those projected values. Runtime code
will convert a projected value to a bounded knob coordinate by subtracting the frozen mean,
dividing by the frozen standard deviation, and applying owned `mathx::tanh`.

This choice keeps the header small: two `f64` statistics per axis plus the squash function
identifier. It also preserves smooth ordering near the semantic center while bounding
outliers without a hard percentile cliff.

## T-52 validation

**Finalized for FORMAT_V1.** After the integrated voice tuning pass, the tanh z-score
squash still lands the semantic axes inside the intended pitch/vowel/contour/warble
ranges without making common tokens feel pinned to the extremes. No artifact regeneration
was needed: the squash function and frozen header statistics from the committed
`assets/format_v1.bin` remain the FORMAT_V1 contract.
