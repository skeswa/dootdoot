# VOICE_V2 Affect Assets

`vader_valence.tsv` is a compact two-column extraction of the VADER sentiment lexicon:

```text
term<TAB>mean_valence
```

Source:

- Repository: `cjhutto/vaderSentiment`
- Commit: `44fc044cd877310ee8278a0eadf34bcd50d41d06`
- Source file: `vaderSentiment/vader_lexicon.txt`
- License: MIT

Only the term and mean valence columns are committed. The standard-deviation and raw
human-rating columns from the upstream file are not needed by dootdoot's deterministic V2
mood planner.

`arousal_signals.toml` is owned project data. It defines deterministic arousal proxy
weights from punctuation density, repeated markers, all-caps, a hand-curated intensifier
list, token count, and character/WordPiece complexity. It does not include AFINN,
SentiWordNet, SUBTLEX-US, NRC-VAD, Warriner, Zipf, or VAD-derived tables.
