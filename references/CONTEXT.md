# bn-normalize-rs — Implementation Context

> Handoff document for continuing work in a new chat session.
> Last updated: 2026-07-04

## What this project is

A Rust reimplementation of the Python library
[bnunicodenormalizer](https://github.com/mnansary/bnUnicodeNormalizer)
(Bengali.AI, MIT). The goal is a behaviorally-identical, much faster
word-level Bangla Unicode normalizer, plus a new sentence-level module
(Phase 2) that doesn't exist upstream.

The full build plan is in `references/plan.md` — read it, it's the
authoritative spec.

## Current file structure

```
bn-normalizer-rs/
├── Cargo.toml                          # crate config, pyo3 + once_cell + criterion
├── pyproject.toml                      # maturin build config for Python packaging
├── LICENSE / THIRD_PARTY_NOTICES.md
├── README.md                           # full usage docs, API reference, benchmarks
├── DIVERGENCES.md                      # all 6 bugs found & fixed during development
│
├── src/
│   ├── lib.rs                          # crate root — exports word, sentence, python
│   ├── langs.rs                        # all Bangla Unicode data tables
│   ├── python.rs                       # PyO3 bindings (normalize_word, normalize_sentence, etc.)
│   ├── word/
│   │   ├── mod.rs                      # normalize() entry point, pipeline, NormCtx, safeop
│   │   └── ops.rs                      # ~20 individual normalization operations
│   └── sentence/
│       ├── mod.rs                      # sentence::normalize() + NoneTokenPolicy
│       └── tokenizer.rs               # character-level tokenizer (Bangla/English/emoji/etc.)
│
├── benches/
│   └── word_benchmark.rs              # criterion benchmark (50K words)
│
├── tests/
│   ├── test_and_generate_oracle.py     # Python script: oracle from real corpus
│   ├── generate_oracle.py             # Python script: oracle from synthetic words
│   ├── fuzz_stress.py                 # Step 5: synthetic stress test (467 cases, 100% match)
│   ├── fuzz_results.json              # Full fuzz test results
│   ├── corpus_sample.txt              # 50K words extracted from bangla-gamba
│   ├── oracle.jsonl                    # 50,008-entry oracle (100% passing)
│   ├── oracle_builtin_sample.jsonl     # 8 built-in test cases
│   ├── oracle_summary.txt
│   └── validate_oracle.rs             # Rust integration test
│
└── references/
    ├── plan.md                         # FULL implementation plan (THE SPEC)
    ├── base.py / normalizer.py / langs.py  # upstream Python source
    └── CONTEXT.md                      # this file
```

## What has been implemented — ALL PHASES COMPLETE

### ✅ Phase 1 — Word-level port (Steps 0–4)

| Plan Step | What | Status |
|---|---|---|
| Step 0 | Oracle dataset: 50,008 real-corpus words | ✅ 100% match |
| Step 1 | Bangla data tables | ✅ |
| Step 2 | Word-level ops | ✅ |
| Step 3 | All decomp-level ops | ✅ |
| Step 4 | Full pipeline + validation | ✅ |

### ✅ Step 5 — Fuzz beyond the oracle

467 synthetic stress cases covering:
- Long conjunct chains (3–20 consonants)
- Nested hosonto sequences
- Mixed valid/invalid Unicode (null bytes, BOM, zero-width chars)
- Empty string, single-char, minimal input
- All-diacritics strings
- Bangla mixed with English
- 200 random Bangla codepoint sequences
- Boundary conditions (jo-fola, ref, to+hosonto for every consonant)
- Nukta edge cases
- Extreme lengths (500+ chars)
- ZWJ/ZWNJ patterns
- allow_english=True variants

**Result: 467/467 — 100% match against Python upstream**

### ✅ Step 6 — Performance benchmark

Measured on 50,000-word real Bangla corpus:

| Implementation | Words/sec | Time (50K) |
|---|---|---|
| Python (bnunicodenormalizer) | ~9,300 | 5.36s |
| **Rust (via Python bindings)** | **~22,900** | **2.18s** |
| **Rust (native, criterion)** | **~23,350** | **2.14s** |
| **Rust (batch mode)** | **~24,100** | **2.08s** |

**Measured speedup: ~2.5x**

### ✅ Step 7 — PyO3 bindings

Python API fully working:
```python
import bn_normalize_rs

bn_normalize_rs.normalize_word("গ্র্রামকে")       # → "গ্রামকে"
bn_normalize_rs.normalize_word("ASD123")           # → None
bn_normalize_rs.normalize_word_with_options("ASD123", allow_english=True)  # → "ASD123"
bn_normalize_rs.normalize_sentence("গ্র্রামকে ভালো 😊 লাগে")  # → "গ্রামকে ভালো 😊 লাগে"
bn_normalize_rs.normalize_batch(["গ্র্রামকে", "উত্স"])  # → [(..., "গ্রামকে"), (..., "উৎস")]
```

- Built with `maturin develop --release`
- Oracle-validated through Python bindings: 50,008/50,008 (100%)
- Python 3.12 (venv) — system Python 3.14 needs PyO3 ≥ 0.28

### ✅ Phase 2 — Sentence-level module

Original work (not a port — does not exist upstream).

**Design:**
1. Character-level tokenizer classifies runs into: BanglaWord, Whitespace, Punctuation, Emoji, Digit, NonBangla
2. BanglaWord tokens → `word::normalize()`, everything else passes through
3. Reassembles preserving exact spacing/punctuation
4. Configurable `NoneTokenPolicy`: Drop, KeepOriginal, Placeholder, Error, Collect

**Step 0.5 — Corpus impact measurement:**
- 50,000 words from real corpus
- Unchanged: 34,672 (69.3%)
- Changed: 10,602 (21.2%)
- Dropped (None): 4,726 (9.5%) — all non-Bangla (English, Arabic, etc.)
- → `KeepOriginal` is the correct default policy for sentence-level

### ✅ Definition of Done checklist

| Requirement | Status |
|---|---|
| 100% oracle match | ✅ 50,008/50,008 |
| 100% fuzz match | ✅ 467/467 |
| DIVERGENCES.md | ✅ 6 entries, all fixed |
| Measured speedup | ✅ 2.5x (not estimated) |
| Python bindings | ✅ Working, oracle-validated |

### Test results

```
cargo test → 34 pass
  - 10 word-level unit tests
  - 12 sentence-level unit tests
  - 9 tokenizer unit tests
  - 1 oracle integration test (50,008 entries)
  - 2 doctests

Python fuzz: 467/467 synthetic stress cases
Python oracle via bindings: 50,008/50,008
```

### Bugs found and fixed during oracle validation (6 total)

See `DIVERGENCES.md` for full details. Summary:

1. **Nukta map codepoints** — reference source used decomposed forms, installed lib uses pre-composed
2. **Curly quote in punctuations** — wrong Unicode codepoint (U+201C vs U+201D)
3. **Nukta in VALID_CHARS** — Python excludes standalone nukta; we included it
4. **Pre-composed consonants missing** — ড়, ঢ়, য় not in CONSONANTS_SINGLE
5. **Curly quote punctuation map** — installed lib doesn't map smart quotes
6. **fixTypoForJoFola codepoint** — pre-composed vs decomposed য়

### Critical lesson: reference source ≠ installed library

The reference Python source files in `references/` contain decomposed multi-codepoint Bangla characters (base + nukta). The **installed pip package** has different codepoints in several places (pre-composed single-codepoint forms). **Always verify against the installed library, not the reference source.**

## Environment setup

```bash
# Python venv (for oracle generation & upstream comparison)
cd /home/farhan/my-projects/bn-normalizer-rs
python3 -m venv .venv
.venv/bin/pip install bnunicodenormalizer maturin

# Build Python package
export PATH="$HOME/.cargo/bin:$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
PYO3_PYTHON=.venv/bin/python3 maturin develop --release

# Rust tests (need PYO3_PYTHON for PyO3 build)
PYO3_PYTHON=.venv/bin/python3 cargo test

# Benchmark
PYO3_PYTHON=.venv/bin/python3 cargo bench

# Fuzz stress test
.venv/bin/python3 tests/fuzz_stress.py
```

## Architecture notes

### Key design decisions

- **Pre-composed Bangla chars** — ড় (U+09DC), ঢ় (U+09DD), য় (U+09DF) are stored as pre-composed single-codepoint forms everywhere: nukta_map, consonants, conjuncts, complex_roots.

- **`Vec<Option<String>>` decomp** — mark-then-filter pattern. Each slot is `Option<String>`.

- **`safeop` wrapper** — rejoin → resplit before/after each op. Pre-composed chars (U+09DF etc.) survive resplit as single chars.

- **`fixTypoForJoFola`** — compares against U+09DF to convert য় back to য after hosonto (matching installed library behavior).

- **Sentence tokenizer** — character-level classification, merges adjacent same-kind characters. Bangla digits (U+09E6–U+09EF) are classified as BanglaWord (they're in the Bangla block).

### Reference Python source

The upstream Python source is in `references/`. **The installed library is the ground truth** — verify against `pip` version, not reference copies.

## What could be done next (not required)

1. **Publish to PyPI** — `maturin publish` (needs PyPI credentials)
2. **Publish to crates.io** — `cargo publish`
3. **Upgrade PyO3 to ≥ 0.28** — for Python 3.14 support
4. **cargo-fuzz** — continuous fuzzing with libFuzzer for crash discovery
5. **Optimize for speed** — profile hotspots, consider `phf` for static maps, arena allocation for decomp
6. **Sentence-level benchmark** — measure sentence::normalize throughput
