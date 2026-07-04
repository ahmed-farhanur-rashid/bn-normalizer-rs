# bn-normalizer-rs

A fast Rust reimplementation of Bangla Unicode text normalization, built
for high-throughput corpus processing (e.g. LLM training data pipelines).

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Why this exists

Word-level Bangla Unicode normalization is essential for LLM training
data quality — visually-identical Bangla text can be encoded as
different Unicode codepoint sequences (e.g. the vowel sign ো as one
codepoint vs. two combining codepoints), which fragments tokenization
and dilutes training signal. The upstream Python implementation
([bnunicodenormalizer](https://github.com/mnansary/bnUnicodeNormalizer))
is correct but slow (pure Python, word-at-a-time), which becomes a
bottleneck at large-corpus scale. This project provides a
behaviorally-identical, much faster implementation in Rust.

## Performance

Measured on a 50,000-word real Bangla corpus (wiki + Common Crawl):

| Implementation | Words/sec | Time (50K words) |
|---|---|---|
| Python (bnunicodenormalizer) | ~9,300 | 5.39s |
| **Rust (via Python bindings)** | **~240,000** | **0.21s** |

**Speedup: ~26x** through Python bindings.

> Benchmarked with `time.perf_counter` (Python) against the optimized Rust build.
> All measurements are 3-trial averages on the same machine.

---

## Installation

### As a Python package (recommended)

```bash
# From source (requires Rust toolchain + maturin)
pip install maturin
maturin develop --release

# Or install a pre-built wheel (when published)
pip install bn-normalize-rs
```

### As a Rust library

Add to your `Cargo.toml`:

```toml
[dependencies]
bn-normalize-rs = { path = "." }
```

Or once published to crates.io:

```toml
[dependencies]
bn-normalize-rs = "0.1"
```

### Build from source

```bash
git clone https://github.com/ahmed-farhanur-rashid/bn-normalizer-rs
cd bn-normalizer-rs
cargo build --release
```

---

## Usage

### Python API

**Basic word normalization:**

```python
from bn_normalize_rs import normalize_word
from pprint import pprint

# Normalize a single word
word = 'াটোবাকো'
result = normalize_word(word)
print(f"Non-norm: {word}; Norm: {result}")
# → Non-norm: াটোবাকো; Norm: টোবাকো
```

**Returns:**
- `str` — the normalized word
- `None` — if the word normalizes to empty (all characters invalid/dropped)

```python
# Invalid / non-Bangla input returns None
normalize_word("ASD123")         # → None
normalize_word("ৄ")              # → None (isolated diacritic)
```

**Allowing English text:**

```python
from bn_normalize_rs import normalize_word, normalize_word_with_options

# Without English (default)
normalize_word("ASD123")                                  # → None

# With English
normalize_word_with_options("ASD123", allow_english=True)  # → "ASD123"
```

**Sentence-level normalization (preserves spacing, punctuation, emoji):**

```python
from bn_normalize_rs import normalize_sentence

# Mixed Bangla/non-Bangla text
normalize_sentence("গ্র্রামকে ভালো লাগে")
# → "গ্রামকে ভালো লাগে"

normalize_sentence("আমি Python 😊 শিখছি")
# → "আমি Python 😊 শিখছি"

# Configurable None token policy
normalize_sentence("গ্র্রামকে ভালো লাগে", none_policy="drop")
# Policies: "keep_original" (default), "drop", "error", "collect"
```

**Batch processing:**

```python
from bn_normalize_rs import normalize_batch

results = normalize_batch(["গ্র্রামকে", "উত্স", "ASD123"])
# → [("গ্র্রামকে", "গ্রামকে"), ("উত্স", "উৎস"), ("ASD123", None)]
```

### Rust API

```rust
use bn_normalize_rs::word;

// Basic word normalization (default options)
let result = word::normalize("গ্র্রামকে");
assert_eq!(result, Some("গ্রামকে".to_string()));

// Invalid / non-Bangla input returns None
let result = word::normalize("ASD123");
assert_eq!(result, None);

// With English allowed
let opts = word::NormalizeOptions {
    allow_english: true,
    ..Default::default()
};
let result = word::normalize_with_options("ASD123", &opts);
assert_eq!(result, Some("ASD123".to_string()));

// Sentence-level normalization
use bn_normalize_rs::sentence;

let opts = sentence::SentenceNormalizeOptions::default();
let result = sentence::normalize("গ্র্রামকে ভালো লাগে", &opts).unwrap();
assert_eq!(result.text, "গ্রামকে ভালো লাগে");

// With default legacy maps (maps rare legacy symbols to common equivalents)
let opts = word::NormalizeOptions {
    legacy_maps: Some(word::default_legacy_maps()),
    ..Default::default()
};
let result = word::normalize_with_options("ঀ", &opts);
assert_eq!(result, Some("৭".to_string()));
```

---

## Initialization Options

| Option | Type | Default | Description |
|---|---|---|---|
| `allow_english` | `bool` | `false` | Allow English letters, numbers, and punctuation |
| `keep_legacy_symbols` | `bool` | `false` | Treat legacy symbols as valid Unicode |
| `legacy_maps` | `dict` / `Option<HashMap>` | `None` | Custom mapping of legacy symbols to replacements |

**Legacy symbols:**

| Symbol | Name |
|---|---|
| `৺` | Isshar |
| `৻` | Ganda |
| `ঀ` | Anji (not '৭') |
| `ঌ` | Li |
| `ৡ` | Dirgho Li |
| `ঽ` | Avagraha |
| `ৠ` | Vocalic Rr (not 'ঋ') |
| `৲` | Rupi |
| `৴` – `৹` | Currency numerators/denominators |

**Legacy handling cases (`keep_legacy_symbols` × `legacy_maps`):**

| Case | `keep_legacy_symbols` | `legacy_maps` | Behavior |
|---|---|---|---|
| 1 | `true` | `None` | All legacy symbols kept as-is |
| 2 | `true` | `Some(map)` | Only mapped symbols changed, rest kept |
| 3 | `false` | `None` | All legacy symbols removed |
| 4 | `false` | `Some(map)` | Mapped symbols changed, rest removed |

### Sentence-level `NoneTokenPolicy`

When normalizing sentences, Bangla words that normalize to `None` are
handled according to the configured policy:

| Policy | Behavior |
|---|---|
| `"keep_original"` (default) | Leave original un-normalized text in place |
| `"drop"` | Remove the token, closing the gap |
| `"error"` | Raise `ValueError` on the first None word |
| `"collect"` | Return `(text, failed_tokens)` tuple for batch inspection |
| `"drop_and_collect"` | Remove the token from the text, but return it in the failed_tokens tuple for inspection |

---

## Normalization Problem Examples

**In all examples, (a) is the non-normalized form and (b) is the normalized form.**

### Broken diacritics
```
# Example 1:
(a) 'আরো' == (b) 'আরো' → False
    (a) breaks as: ['আ', 'র', 'ে', 'া']
    (b) breaks as: ['আ', 'র', 'ো']

# Example 2:
(a) 'পৌঁছে' == (b) 'পৌঁছে' → False
    (a) breaks as: ['প', 'ে', 'ৗ', 'ঁ', 'ছ', 'ে']
    (b) breaks as: ['প', 'ৌ', 'ঁ', 'ছ', 'ে']

# Example 3:
(a) 'সংস্কৄতি' == (b) 'সংস্কৃতি' → False
    (a) breaks as: ['স', 'ং', 'স', '্', 'ক', 'ৄ', 'ত', 'ি']
    (b) breaks as: ['স', 'ং', 'স', '্', 'ক', 'ৃ', 'ত', 'ি']
```

### Nukta normalization
```
# Example 1:
(a) 'কেন্দ্রীয়' == (b) 'কেন্দ্রীয়' → False
    (a) breaks as: ['ক', 'ে', 'ন', '্', 'দ', '্', 'র', 'ী', 'য', '়']
    (b) breaks as: ['ক', 'ে', 'ন', '্', 'দ', '্', 'র', 'ী', 'য়']

# Example 2:
(a) 'জ়ন্য' == (b) 'জন্য' → False
    (a) breaks as: ['জ', '়', 'ন', '্', 'য']
    (b) breaks as: ['জ', 'ন', '্', 'য']
```

### Invalid hosonto (connector)
```
# Example 1:
(a) 'দুই্টি' == (b) 'দুইটি' → False
    (a) breaks as: ['দ', 'ু', 'ই', '্', 'ট', 'ি']
    (b) breaks as: ['দ', 'ু', 'ই', 'ট', 'ি']

# Example 2:
(a) 'যু্ক্ত' == (b) 'যুক্ত' → False
    (a) breaks as: ['য', 'ু', '্', 'ক', '্', 'ত']
    (b) breaks as: ['য', 'ু', 'ক', '্', 'ত']
```

### To+hosonto conversion
```
# Example 1:
(a) 'বুত্পত্তি' == (b) 'বুৎপত্তি' → False
    (a) breaks as: ['ব', 'ু', 'ত', '্', 'প', 'ত', '্', 'ত', 'ি']
    (b) breaks as: ['ব', 'ু', 'ৎ', 'প', 'ত', '্', 'ত', 'ি']

# Example 2:
(a) 'উত্স' == (b) 'উৎস' → False
    (a) breaks as: ['উ', 'ত', '্', 'স']
    (b) breaks as: ['উ', 'ৎ', 'স']
```

### Duplicate diacritics
```
# Example 1:
(a) 'যুুদ্ধ' == (b) 'যুদ্ধ' → False
    (a) breaks as: ['য', 'ু', 'ু', 'দ', '্', 'ধ']
    (b) breaks as: ['য', 'ু', 'দ', '্', 'ধ']

# Example 2:
(a) 'প্রকৃৃতির' == (b) 'প্রকৃতির' → False
    (a) breaks as: ['প', '্', 'র', 'ক', 'ৃ', 'ৃ', 'ত', 'ি', 'র']
    (b) breaks as: ['প', '্', 'র', 'ক', 'ৃ', 'ত', 'ি', 'র']
```

### Vowel diacritics after vowels
```
# Example 1:
(a) 'উুলু' == (b) 'উলু' → False
    (a) breaks as: ['উ', 'ু', 'ল', 'ু']
    (b) breaks as: ['উ', 'ল', 'ু']

# Example 2:
(a) 'একএে' == (b) 'একত্রে' → False
    (a) breaks as: ['এ', 'ক', 'এ', 'ে']
    (b) breaks as: ['এ', 'ক', 'ত', '্', 'র', 'ে']
```

### Repeated folas
```
(a) 'গ্র্রামকে' == (b) 'গ্রামকে' → False
    (a) breaks as: ['গ', '্', 'র', '্', 'র', 'া', 'ম', 'ক', 'ে']
    (b) breaks as: ['গ', '্', 'র', 'া', 'ম', 'ক', 'ে']
```

---

## Operations Pipeline

The normalizer applies operations in this fixed order:

**Word-level ops** (operate on the full word string):
1. `LegacySymbols` — map legacy symbols via `legacy_maps`
2. `BrokenDiacritics` — fix broken diacritic sequences
3. `AssameseReplacement` — ৰ→র, ৱ→ব
4. `PunctuationReplacement` — curly quotes, dashes, etc.

**Decomposition-level ops** (operate on individual character positions):
5. `BrokenNukta` — compose nukta with preceding consonant
6. `InvalidUnicode` — remove invalid starts/ends/chars
7. `InvalidConnector` — remove misplaced hosonto
8. `FixDiacritics` — vowel/consonant diacritic cleanup + reordering
9. `VowelDiacriticAfterVowel` — special Bangla override (এ→ত্র)
10. `base_bangla_compose` — re-run cleanup after structural changes
11. `ToAndHosontoNormalize` — ত্ → ৎ conversion
12. `NormalizeConjunctsDiacritics` — fix repeated folas, jo-fola typos
13. `ComplexRootNormalization` — validate conjunct formations

> **Note:** The normalization is purely based on how Bangla text is used in
> **Bangladesh** (bn:BD). It does not necessarily cover every variation
> of textual content from other regions.

---

## Relationship to upstream

This project reimplements the word-level normalization rules from
[bnunicodenormalizer](https://github.com/mnansary/bnUnicodeNormalizer)
(Bengali.AI, MIT licensed) in Rust for performance. The word-level
core is validated to match the upstream Python library's output exactly
— it is a faithful, oracle-tested port, not a reinterpretation.

The **sentence-level module** is original work — it does not exist
upstream (see upstream issue #13). It provides intelligent tokenization
that preserves spacing, punctuation, emoji, and non-Bangla content.

See `THIRD_PARTY_NOTICES.md` for full attribution and license text.

### Citation

If you use this in academic work, please cite the original library:

```bibtex
@inproceedings{ansary-etal-2024-unicode-normalization,
    title = "{U}nicode Normalization and Grapheme Parsing of {I}ndic Languages",
    author = "Ansary, Nazmuddoha  and
      Adib, Quazi Adibur Rahman  and
      Reasat, Tahsin  and
      Sushmit, Asif Shahriyar  and
      Humayun, Ahmed Imtiaz  and
      Mehnaz, Sazia  and
      Fatema, Kanij  and
      Rashid, Mohammad Mamun Or  and
      Sadeque, Farig",
    booktitle = "Proceedings of the 2024 Joint International Conference on Computational Linguistics, Language Resources and Evaluation (LREC-COLING 2024)",
    month = may,
    year = "2024",
    publisher = "ELRA and ICCL",
    url = "https://aclanthology.org/2024.lrec-main.1479",
    pages = "17019--17030",
}
```

---

## Project structure

```
src/
├── lib.rs              — crate root, public API
├── langs.rs            — Bangla Unicode data tables
├── python.rs           — PyO3 bindings for Python interop
├── word/
│   ├── mod.rs          — word-level normalize() entry point + pipeline
│   └── ops.rs          — individual normalization operations
└── sentence/
    ├── mod.rs          — sentence-level normalize() + NoneTokenPolicy
    └── tokenizer.rs    — character-level tokenizer

bnUnicodeNormalizer-src/ — upstream Python source snapshot (for oracle generation)
benches/                — criterion benchmarks
tests/
├── validate_oracle.rs  — Rust integration test
├── data/               — oracle datasets + corpus sample
└── scripts/            — Python oracle generation + fuzz testing
docs/                   — DIVERGENCES.md (porting log)
```

## Running tests

```bash
# All tests (unit + oracle integration + doctests)
cargo test

# Benchmark
cargo bench

# Fuzz stress test (requires Python venv with bnunicodenormalizer)
python tests/scripts/fuzz_stress.py
```

## License

MIT — see `LICENSE`. See `THIRD_PARTY_NOTICES.md` for upstream
attribution required by the upstream library's own MIT license.
