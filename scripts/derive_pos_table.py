# /// script
# requires-python = ">=3.11,<3.13"
# dependencies = [
#   "click>=8,<9",
#   "numpy>=1.26,<2",
#   "pyarrow>=16,<21",
#   "spacy>=3.7,<3.8",
#   "en-core-web-sm @ https://github.com/explosion/spacy-models/releases/download/en_core_web_sm-3.7.1/en_core_web_sm-3.7.1-py3-none-any.whl",
# ]
# [tool.uv]
# exclude-newer = "2026-06-01T00:00:00Z"
# ///
"""One-time derivation of the VOICE_V12 POS tagged-counts snapshot (T-120).

Downloads one pinned parquet shard of the public CommitChronicle corpus
(JetBrains-Research/commit-chronicle, ASE 2023 — commits collected from
permissively-licensed [Apache/BSD-3/MIT] GitHub repositories), verifies its
SHA-256, POS-tags the commit *messages* with the MIT-licensed spaCy
``en_core_web_sm`` model, and writes aggregate word statistics to
``assets/pos/tagged_counts.tsv``.

Only derived statistics are committed — surface form, lemma, coarse POS bucket,
and count. No message text is redistributed. The committed TSV is the pinned
"ranking-corpus snapshot" of FR-114: its SHA-256 goes into
``assets/source_manifest.toml`` and ``xtask`` validates it before deriving the
baked class table (the classification *policy* — dominant class, closed-class
override, conservative ambiguity fallback, surface expansion — lives in
``xtask`` where it is unit-tested).

Run via the locked uv environment so the tagger is reproducible:

    uv run scripts/derive_pos_table.py

Determinism notes: rows are exact counts sorted lexicographically; spaCy is
version-pinned by the lock file; the corpus shard is hash-pinned below. Re-runs
produce byte-identical output for the same pins.
"""

from __future__ import annotations

import hashlib
import re
import sys
import urllib.request
from collections import Counter
from pathlib import Path

CORPUS_HF_REPO = "JetBrains-Research/commit-chronicle"
CORPUS_REVISION = "5fd076e67b812a9f3d1999e5e40f71715f84bb51"
CORPUS_FILE = "data/test-00000-of-00012-2085aa4b49c438e4.parquet"
CORPUS_SHA256 = "b05b4ab34973c358d18475173e301ba0534cb0f23a15f91ba07b3af3fbf0d988"
SPACY_MODEL = "en_core_web_sm"
SPACY_MODEL_VERSION = "3.7.1"

REPO_ROOT = Path(__file__).resolve().parent.parent
CACHE_DIR = REPO_ROOT / "target" / "pos-corpus"
OUTPUT_PATH = REPO_ROOT / "assets" / "pos" / "tagged_counts.tsv"

# Keep the committed snapshot focused: lemmas ranked by content (noun+verb)
# usage, with every attested surface form of a kept lemma.
TOP_LEMMAS = 6_000
MIN_SURFACE_COUNT = 3

WORD_RE = re.compile(r"^[a-z][a-z]{1,19}$")
URL_RE = re.compile(r"https?://\S+")
TRAILER_RE = re.compile(
    r"^\s*(signed-off-by|co-authored-by|reviewed-by|acked-by|tested-by|"
    r"reported-by|suggested-by|cc|fixes|closes|resolves|see-also|change-id|"
    r"bug|issue|pr-url|refs?)\s*:",
    re.IGNORECASE,
)


def download_corpus() -> Path:
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    target = CACHE_DIR / Path(CORPUS_FILE).name

    if not target.exists():
        url = (
            f"https://huggingface.co/datasets/{CORPUS_HF_REPO}/resolve/"
            f"{CORPUS_REVISION}/{CORPUS_FILE}"
        )
        print(f"downloading {url}", file=sys.stderr)
        with urllib.request.urlopen(url) as response, target.open("wb") as out:
            while chunk := response.read(1 << 20):
                out.write(chunk)

    digest = hashlib.sha256(target.read_bytes()).hexdigest()
    if digest != CORPUS_SHA256:
        target.unlink()
        raise SystemExit(
            f"corpus shard sha256 mismatch: expected {CORPUS_SHA256}, got {digest}"
        )

    return target


def load_messages(parquet_path: Path) -> list[str]:
    import pyarrow.parquet as pq

    table = pq.read_table(parquet_path, columns=["message"])
    messages = []
    for value in table.column("message").to_pylist():
        if not value:
            continue
        lines = [
            line
            for line in value.splitlines()
            if line.strip() and not TRAILER_RE.match(line)
        ]
        cleaned = URL_RE.sub(" ", "\n".join(lines)).strip()
        if cleaned:
            messages.append(cleaned)

    return messages


def tag_messages(messages: list[str]) -> Counter[tuple[str, str, str]]:
    import spacy

    nlp = spacy.load(SPACY_MODEL, disable=["parser", "ner"])
    if nlp.meta["version"] != SPACY_MODEL_VERSION:
        raise SystemExit(
            f"{SPACY_MODEL} version mismatch: expected {SPACY_MODEL_VERSION}, "
            f"got {nlp.meta['version']}"
        )

    counts: Counter[tuple[str, str, str]] = Counter()
    total = len(messages)

    for index, doc in enumerate(nlp.pipe(messages, batch_size=256)):
        if index % 10_000 == 0:
            print(f"tagged {index}/{total}", file=sys.stderr)
        for token in doc:
            surface = token.text.lower()
            lemma = token.lemma_.lower()
            if not WORD_RE.match(surface) or not WORD_RE.match(lemma):
                continue
            if token.pos_ == "NOUN":
                bucket = "noun"
            elif token.pos_ == "VERB":
                bucket = "verb"
            else:
                # PROPN, AUX, ADJ, closed classes, … — everything that is not a
                # common noun or lexical verb counts against content dominance.
                bucket = "other"
            counts[(surface, lemma, bucket)] += 1

    return counts


def write_snapshot(counts: Counter[tuple[str, str, str]]) -> None:
    lemma_content: Counter[str] = Counter()
    for (_surface, lemma, bucket), count in counts.items():
        if bucket in ("noun", "verb"):
            lemma_content[lemma] += count

    kept_lemmas = {lemma for lemma, _count in lemma_content.most_common(TOP_LEMMAS)}
    rows = [
        (surface, lemma, bucket, count)
        for (surface, lemma, bucket), count in counts.items()
        if lemma in kept_lemmas and count >= MIN_SURFACE_COUNT
    ]
    rows.sort()

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with OUTPUT_PATH.open("w", encoding="utf-8", newline="\n") as out:
        out.write(
            "# VOICE_V12 POS tagged-counts snapshot (derived statistics only).\n"
            f"# corpus: hf.co/datasets/{CORPUS_HF_REPO}@{CORPUS_REVISION}\n"
            f"#   file: {CORPUS_FILE} (sha256 {CORPUS_SHA256})\n"
            "#   CommitChronicle (ASE 2023): commits from permissively-licensed\n"
            "#   (Apache/BSD-3/MIT) GitHub repositories. No message text is\n"
            "#   reproduced here - only per-word aggregate counts.\n"
            f"# tagger: spaCy {SPACY_MODEL} {SPACY_MODEL_VERSION} (MIT)\n"
            f"# derivation: uv run scripts/derive_pos_table.py (top {TOP_LEMMAS}\n"
            f"#   content lemmas, surface count >= {MIN_SURFACE_COUNT})\n"
            "# columns: surface<TAB>lemma<TAB>bucket(noun|verb|other)<TAB>count\n"
        )
        for surface, lemma, bucket, count in rows:
            out.write(f"{surface}\t{lemma}\t{bucket}\t{count}\n")

    digest = hashlib.sha256(OUTPUT_PATH.read_bytes()).hexdigest()
    print(f"wrote {OUTPUT_PATH} ({len(rows)} rows)")
    print(f"tagged_counts_sha256 = \"{digest}\"")


def main() -> None:
    parquet_path = download_corpus()
    messages = load_messages(parquet_path)
    print(f"loaded {len(messages)} messages", file=sys.stderr)
    counts = tag_messages(messages)
    write_snapshot(counts)


if __name__ == "__main__":
    main()
