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
| Python (bnunicodenormalizer) | ~9,300 | 5.36s |
| **Rust (via Python bindings)** | **~22,900** | **2.18s** |
| **Rust (native)** | **~23,350** | **2.14s** |

**Speedup: ~2.5x** through Python bindings, ~2.5x native Rust.

> Benchmarked with criterion (Rust) and time.perf_counter (Python).
> All measurements are 3-trial averages on the same machine.

## Installation

### As a Python package (recommended)

```bash
# Build and install from source
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

## Usage

### Python API

```python
import bn_normalize_rs

# Word-level normalization
result = bn_normalize_rs.normalize_word("গ্র্রামকে")   # → "গ্রামকে"
result = bn_normalize_rs.normalize_word("ASD123")       # → None

# With options
result = bn_normalize_rs.normalize_word_with_options(
    "ASD123", allow_english=True
)  # → "ASD123"

# Sentence-level normalization (preserves spacing, punctuation, emoji)
result = bn_normalize_rs.normalize_sentence("গ্র্রামকে ভালো লাগে")
# → "গ্রামকে ভালো লাগে"

result = bn_normalize_rs.normalize_sentence("আমি Python 😊 শিখছি")
# → "আমি Python 😊 শিখছি"

# Batch processing
results = bn_normalize_rs.normalize_batch(["গ্র্রামকে", "উত্স", "ASD123"])
# → [("গ্র্রামকে", "গ্রামকে"), ("উত্স", "উৎস"), ("ASD123", None)]

# Sentence with configurable None policy
result = bn_normalize_rs.normalize_sentence(
    "গ্র্রামকে ভালো লাগে",
    none_policy="drop"          # or "keep_original", "error", "collect"
)
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

### Configuration Options

| Option | Type | Default | Description |
|---|---|---|---|
| `allow_english` | `bool` | `false` | Allow English letters, numbers, and punctuation |
| `keep_legacy_symbols` | `bool` | `false` | Treat legacy symbols as valid Unicode |
| `legacy_maps` | `Option<HashMap<char, String>>` | `None` | Custom mapping of legacy symbols to replacements |

**Legacy symbol handling follows the same rules as upstream:**

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

## What it normalizes

This library handles the following classes of Bangla Unicode issues:

- **Broken diacritics** — e.g. `আরো` (ে+া) → `আরো` (ো)
- **Nukta normalization** — e.g. `য` + `়` → `য়`
- **Invalid hosonto (connector)** — e.g. `দুই্টি` → `দুইটি`
- **To+hosonto conversion** — e.g. `উত্স` → `উৎস`
- **Duplicate diacritics** — e.g. `যুুদ্ধ` → `যুদ্ধ`
- **Vowel diacritics after vowels** — e.g. `উুলু` → `উলু`
- **Repeated folas** — e.g. `গ্র্রামকে` → `গ্রামকে`
- **Complex root normalization** — validates and fixes conjunct formations
- **Assamese character replacement** — ৰ→র, ৱ→ব
- **Punctuation normalization** — curly quotes, dashes, etc.

> **Note:** The normalization is based on how Bangla text is used in
> **Bangladesh** (bn:BD). It does not necessarily cover every variation
> of textual content from other regions.

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

## Project structure

```
src/
├── lib.rs              — crate root, public API
├── langs.rs            — Bangla Unicode data tables (vowels, consonants,
│                         diacritics, conjuncts, normalization maps)
├── python.rs           — PyO3 bindings for Python interop
├── word/
│   ├── mod.rs          — word-level normalize() entry point + pipeline
│   └── ops.rs          — individual normalization operations
└── sentence/
    ├── mod.rs          — sentence-level normalize() + NoneTokenPolicy
    └── tokenizer.rs    — character-level tokenizer (Bangla/English/emoji/etc.)

benches/                — criterion benchmarks
references/             — upstream Python source snapshot (for oracle generation)
tests/                  — oracle validation (50,008 words, 100% match)
THIRD_PARTY_NOTICES.md  — upstream license + attribution
LICENSE                 — this project's MIT license
```

## Running tests

```bash
# All tests (unit + oracle integration + doctests)
cargo test

# Benchmark
cargo bench
```

## License

MIT — see `LICENSE`. See `THIRD_PARTY_NOTICES.md` for upstream
attribution required by the upstream library's own MIT license.
