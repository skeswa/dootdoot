# Squash Function

`FORMAT_V1` uses a tanh z-score squash for projected PCA values.

For each retained PCA axis, `xtask` projects every token embedding, then records the
population mean and population standard deviation of those projected values. Runtime code
will convert a projected value to a bounded knob coordinate by subtracting the frozen mean,
dividing by the frozen standard deviation, and applying owned `mathx::tanh`.

This choice keeps the header small: two `f64` statistics per axis plus the squash function
identifier. It also preserves smooth ordering near the semantic center while bounding
outliers without a hard percentile cliff.
