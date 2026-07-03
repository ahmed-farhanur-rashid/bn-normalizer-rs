# bn-normalize-rs — Implementation Context

> Handoff document for continuing work in a new chat session.
> Last updated: 2026-07-03

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
├── Cargo.toml                          # crate config, depends on once_cell
├── LICENSE / THIRD_PARTY_NOTICES.md
├── README.md                           # usage docs, API reference
│
├── src/
│   ├── lib.rs                          # crate root — exports `word` and `langs`
│   ├── langs.rs                        # all Bangla Unicode data tables
│   └── word/
│       ├── mod.rs                      # normalize() entry point, pipeline, NormCtx, safeop
│       └── ops.rs                      # ~20 individual normalization operations
│
├── tests/
│   ├── test_and_generate_oracle.py     # Python script: oracle from real corpus
│   ├── generate_oracle.py             # Python script: oracle from synthetic words
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

## What has been implemented (Phase 1 — COMPLETE)

### ✅ Fully validated

| Plan Step | What | Status |
|---|---|---|
| Step 0 | Oracle dataset: 50,008 real-corpus words | ✅ 100% match |
| Step 1 | Bangla data tables | ✅ |
| Step 2 | Word-level ops | ✅ |
| Step 3 | All decomp-level ops | ✅ |
| Step 4 | Full pipeline + validation | ✅ |

### Test results

```
cargo test → 12/12 pass
  - 10 unit tests (upstream README examples)
  - 1 oracle integration test (50,008 entries from real Bangla wiki+CC corpus)
  - 1 doctest
```

### Bugs found and fixed during oracle validation (6 total)

1. **Nukta map codepoints** — Python's installed `nukta_map` uses pre-composed single-codepoint forms (U+09DC ড়, U+09DD ঢ়, U+09DF য়). Our source literals used decomposed (base+nukta) forms. Fixed to use pre-composed escape sequences.

2. **Curly quote in punctuations** — Python's punctuations has `"` (U+201D, RIGHT/closing), not `"` (U+201C, LEFT/opening). Fixed all 4 sets.

3. **Nukta in VALID_CHARS** — Python does NOT include standalone nukta (U+09BC) in valid. Removed.

4. **Pre-composed consonants missing** — Added U+09DC, U+09DD, U+09DF to CONSONANTS_SINGLE so they're in VALID_CHARS and CONSONANTS_SINGLE_SET.

5. **Curly quote punctuation map** — The installed library does NOT map `'`/`'` to straight quotes. Removed those entries.

6. **fixTypoForJoFola codepoint mismatch** — The installed library compares against U+09DF (pre-composed, 1 codepoint), not the decomposed `'য়'` (2 codepoints) from the reference source. Also fixed the conjuncts table entry for `য়্য` to use pre-composed form.

### Critical lesson: reference source ≠ installed library

The reference Python source files in `references/` contain decomposed multi-codepoint Bangla characters (base + nukta). The **installed pip package** has different codepoints in several places (pre-composed single-codepoint forms). **Always verify against the installed library, not the reference source.**

## What to implement next (in order)

Read `references/plan.md` for full spec. Below is what's left and how to do it.

### 1. Benchmarking (Plan Step 6 — do this first, it's quick)

Measure Rust vs Python words/sec on the 50K corpus:

```bash
# Rust (release mode):
cargo bench   # or write a simple bench in tests/bench.rs using std::time

# Python baseline:
.venv/bin/python3 -c "
from bnunicodenormalizer import Normalizer
import time
n = Normalizer()
words = open('tests/corpus_sample.txt').read().splitlines()
t0 = time.time()
for w in words: n(w)
print(f'{len(words)/(time.time()-t0):.0f} words/sec')
"
```

Consider using `criterion` crate for proper benchmarking. Add to `Cargo.toml`:
```toml
[dev-dependencies]
criterion = "0.5"
```

### 2. Fuzz testing (Plan Step 5 — optional but recommended)

Use `cargo-fuzz` to generate random Bangla codepoint sequences and check for panics:
```bash
cargo install cargo-fuzz
cargo fuzz init
# Create fuzz target that calls word::normalize() on arbitrary &str
```

Focus on: random sequences from U+0980–U+09FF range, edge cases like all-nukta, all-hosonto, empty strings, single chars.

### 3. PyO3 bindings (Plan Step 7 — HIGH priority)

Create a Python-callable wrapper so this can be a drop-in replacement in existing pipelines.

**Files to create:**
- `Cargo.toml` — add `pyo3` dependency with `extension-module` feature
- `src/python.rs` — PyO3 module exposing `normalize(word)` and `normalize_with_options(word, allow_english, keep_legacy_symbols)`
- `pyproject.toml` — for `maturin` build system
- Update `src/lib.rs` to conditionally compile the `python` module

**Key design from plan.md:**
```rust
// Word-level: matches Python's Normalizer()(word)["normalized"]
#[pyfunction]
fn normalize(word: &str) -> Option<String> { ... }
```

**Build & test:**
```bash
pip install maturin
maturin develop --release
python3 -c "from bn_normalize_rs import normalize; print(normalize('গ্র্রামকে'))"
```

### 4. Phase 2 — Sentence-level module (NEW WORK, not a port)

This does NOT exist in the upstream Python library. It's original design per `plan.md` section "Phase 2".

**Files to create:**
- `src/sentence/mod.rs` — main `sentence::normalize()` function
- `src/sentence/tokenizer.rs` — classify char runs into: Bangla word, punctuation, whitespace, emoji, digit, non-Bangla word
- Update `src/lib.rs` to export `sentence` module

**Core design (from plan.md):**
1. **Tokenize** input preserving exact structure (NOT `str.split()`)
2. **Classify** each token: Bangla word / punctuation / whitespace / emoji / digit / non-Bangla
3. **Route**: Bangla words → `word::normalize()`, everything else → pass through
4. **Reassemble** preserving original spacing/punctuation

**`NoneTokenPolicy` enum** (configurable, not hardcoded):
```rust
pub enum NoneTokenPolicy {
    Drop,                  // remove token, close gap
    KeepOriginal,          // leave un-normalized text in place
    Placeholder(String),   // replace with marker
    Error,                 // return Err
    Collect,               // recommended default for batch processing
}
```

**Step 0.5 (BEFORE writing Phase 2 code):** Run normalizer over a real corpus sample to measure: % words changed, % dropped to None, spot-check which words are dropped. This informs the default policy.

**Phase 2 test suite:** Hand-built (no oracle — no upstream ground truth). Build from real sentences with emoji, punctuation, mixed Bangla/English, URLs. Expected output is hand-decided.

## Environment setup

```bash
# Python venv (for oracle generation & upstream comparison)
cd /home/farhan/my-projects/bn-normalizer-rs
python3 -m venv .venv
.venv/bin/pip install bnunicodenormalizer

# Rust
cargo test  # should show 12/12 pass

# Re-generate oracle (if needed)
.venv/bin/python3 tests/test_and_generate_oracle.py tests/corpus_sample.txt 50000
cargo test --test validate_oracle
```

## Architecture notes

### Key design decisions

- **Pre-composed Bangla chars** — ড় (U+09DC), ঢ় (U+09DD), য় (U+09DF) are stored as pre-composed single-codepoint forms everywhere: nukta_map, consonants, conjuncts, complex_roots.

- **`Vec<Option<String>>` decomp** — mark-then-filter pattern. Each slot is `Option<String>`.

- **`safeop` wrapper** — rejoin → resplit before/after each op. Pre-composed chars (U+09DF etc.) survive resplit as single chars.

- **`fixTypoForJoFola`** — compares against U+09DF to convert য় back to য after hosonto (matching installed library behavior).

### Reference Python source

The upstream Python source is in `references/`. **The installed library is the ground truth** — verify against `pip` version, not reference copies.
