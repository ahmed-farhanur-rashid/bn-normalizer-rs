# Plan: Rust port of `bnunicodenormalizer`

## Project naming

Repo/crate name: **`bn-normalize-rs`** (kebab-case, standard Rust/crates.io
convention). Do not name it after the upstream repo verbatim (e.g. not
`bnUnicodeNormalizer-rust`) — this is a reimplementation credited via
attribution, not an official fork or port, and the name shouldn't imply
otherwise. Use the same name consistently across: GitHub repo, Cargo
crate (`Cargo.toml` `name`), and PyPI package name if published
(`bn-normalize-rs` or `bn_normalize_rs` per PyPI normalization rules).

## Relationship to upstream — confirmed via their GitHub issues

Checked the upstream repo's issue tracker
(https://github.com/mnansary/bnUnicodeNormalizer) before finalizing
scope. Two issues directly shape this plan:

- **Issue #13** ("Add normalization for sentence"): the maintainer
  confirms explicitly that the library is word-level only, by design —
  sentence/document-level normalization is unsolved even upstream,
  because naive `word.split()` breaks on punctuation and interacts
  badly with NFKD during tokenization. He gives a fragile naive
  workaround and says proper support needs "matrix ops rather than for
  loop operations" — i.e. this is an open, real problem, not a solved
  one we're just failing to find.
- **Issue #9**: emoji are returned as `None` (dropped) by the current
  word-level API, since it assumes Bangla-script input. A community PoC
  fix was mentioned but never landed upstream.

Implication for this plan: the core Rust port must faithfully replicate
word-level-only behavior (see Non-negotiable ground truth rule below) —
do not build sentence-splitting into the core port itself, since there
is no upstream ground truth for it. Sentence-level handling is real,
wanted, and worth building — but it is new work, not a port, and gets
its own module and its own hand-built test suite (see Phase 2).

## Licensing and attribution (MIT — confirmed)

Upstream is MIT licensed (Copyright (c) 2022 Bengali.AI). This permits:
copying their data tables/rules verbatim, reimplementing freely,
relicensing this code under MIT or anything else, commercial use, no
obligation to contribute back or ask permission.

The one real requirement: preserve their copyright notice and license
text somewhere in this repo. Concretely:
- Add `THIRD_PARTY_NOTICES.md` at repo root containing the verbatim MIT
  license text + copyright line from upstream, plus a plain-language
  note explaining what was derived from their work (normalization rules,
  conjunct tables, test cases) vs. what's original to this repo (the
  sentence-level module, Phase 2).
- Add a short "Relationship to upstream" section in the main `README.md`
  linking to https://github.com/mnansary/bnUnicodeNormalizer and
  crediting Bengali.AI, distinct from the full legal text in
  `THIRD_PARTY_NOTICES.md`.
- Do not include their repo as a submodule or copy of their full repo
  structure in this project — only the reference source snapshot
  described below, kept for oracle-generation purposes.
- This repo's own code is your own name/copyright, your own license
  choice (MIT recommended for ecosystem compatibility).

## Status check (done before writing this plan)

Searched GitHub/crates.io — **no existing Rust port of this library exists**.
`unicode-normalization` (crates.io) is generic NFC/NFD only; it does not
implement any of the Bangla-specific rules below. This would be a new,
publishable artifact, not a duplicate of existing work.

Original library: `bnunicodenormalizer` v0.1.7 (PyPI), by Bengali.AI /
BUET CSE NLP. Source copied into `reference_python_source/` next to this
plan — that copy, not memory or re-fetching, is the ground truth for the
port. Do not paraphrase behavior from the README; read the actual code.

Total source: ~1,350 lines across 4 files. This is small and mechanical
(mostly index-walking over a decomposed character list, plus static
lookup tables), not 350 independent "rules" needing individual design
judgment — the conjuncts table alone accounts for ~350 of the perceived
rule count and is a pure data table, not logic.

## Non-negotiable ground truth rule

**The Python implementation is the spec. There is no other spec.**
Docstrings, README examples, and linguistic intuition about Bangla are
useful for understanding *why* a rule exists, but when they conflict with
what the code actually does, the code wins. Do not "fix" perceived bugs
in the original during the port — replicate them exactly, byte for byte.
If a rule looks wrong, flag it in a NOTES.md rather than silently
correcting it. A faster library that's behaviorally different from the
one everyone benchmarks against is not a valid replacement.

## Architecture of the original (read this before writing Rust)

Pipeline per single word (`Normalizer.__call__` in `normalizer.py`):

1. **Word-level ops** (operate on the raw Python string, dict-ordered):
   - `LegacySymbols` — string `.replace()` via `legacy_maps` dict (optional, user-configured)
   - `BrokenDiacritics` — string `.replace()` via `diacritic_map` (fixes visually-identical-but-different-codepoint diacritics, e.g. two different encodings of `ো`)
   - `AssameseReplacement`, `PunctuationReplacement` (Bangla subclass adds these)
2. **Decompose**: `self.decomp = [ch for ch in self.word]` — a list of individual Python characters (i.e. Unicode scalar values / codepoints, NOT grapheme clusters).
3. **Decomp-level ops**, run in this exact order (Python dict preserves insertion order — order is semantically load-bearing):
   - `BrokenNukta` → `fixBrokenNukta`
   - `InvalidUnicode` → `cleanInvalidUnicodes`
   - `InvalidConnector` → `cleanInvalidConnector`
   - `FixDiacritics` → `cleanDiacritics` (itself 4 sub-steps)
   - `VowelDiacriticAfterVowel` → `cleanVowelDiacriticComingAfterVowel`
   - (Bangla subclass appends 3 more): `base_bangla_compose`, `ToAndHosontoNormalize`, `NormalizeConjunctsDiacritics`, `ComplexRootNormalization`
   - Each op is wrapped in `safeop()`, which: strips `None` placeholders, **rejoins the list into a string and re-splits into chars** (this matters if any op ever produces a multi-codepoint string, which resets indices), runs the op, strips `None` again, rejoins/resplits again.
4. **Final compose**: `baseCompose()` runs a fixed sub-pipeline again (`cleanInvalidUnicodes`, `cleanInvalidConnector`, `cleanDiacritics`, `cleanVowelDiacriticComingAfterVowel`, `fixNoSpaceChar`), each also via `safeop`.
5. Join `decomp` back into a string, filtering `None`.
6. If at any point `decomp` becomes empty, the word normalizes to `None` (word is invalid/dropped, not just unchanged).

Every op function mutates `self.decomp` in place, mostly by:
- Walking `for idx, d in enumerate(self.decomp)`
- Looking at fixed offsets (`self.decomp[idx+1]`, `idx-1`, up to `idx+4` in `convertToAndHosonto`)
- Either setting `self.decomp[idx] = None` (delete), replacing it with another char, or swapping two adjacent entries (`swapIdxs`)

**Important subtlety**: several ops mutate the list while iterating over
the *original* enumeration (Python's `enumerate` doesn't see `None`
writes retroactively change indices — the list is mutated but not
resized until `safeop` filters `None`s afterward). A naive Rust port
using something like `Vec::retain` mid-loop instead of "mark None, filter
after" will desync index offsets and silently corrupt multi-character
patterns (e.g. the double-fola / repeated-conjunct cleanup, or the
4-lookahead in `convertToAndHosonto`). **Port the mark-then-filter
pattern exactly; do not "clean it up" into in-place removal.**

Static data (all in `langs.py`, per-language, only `bangla` needed):
vowels, consonants, vowel/consonant diacritics, connector (`্`),
`nukta`, `nukta_map`, `diacritic_map`, `conjuncts` (~350 strings),
`complex_roots`, `legacy_symbols`, `legacy_maps`, `invalid_starts`,
`invalid_connectors`, `non_chars`, punctuation/number lists. These are
just `HashSet<char>` / `HashSet<String>` / `HashMap<char,char>` /
`HashMap<char,String>` in Rust — no logic, just data. Port these first,
they're zero-risk.

`indic.py` implements `IndicNormalizer` for other Indic scripts
(Devanagari, Gujarati, etc.) — **out of scope for this port**. Only
Bangla is needed. Confirm this with the user before spending any time on
it; do not port it speculatively.

## Deliverable shape

One crate, `bn-normalize-rs`, two modules:

```rust
// core module — faithful port, oracle-validated against upstream Python
mod word {
    pub fn normalize(word: &str) -> Option<String>;
}

// new module — original work, not present upstream, own test suite (Phase 2)
mod sentence {
    pub fn normalize(text: &str, opts: SentenceNormalizeOptions) -> SentenceNormalizeResult;
}
```

`word::normalize` matches Python's `Normalizer()(word)["normalized"]`
behavior exactly (including `None` on drop). The `"ops"` audit trail
from the Python version is nice-to-have for debugging but not required
for the core deliverable — do not spend time on it before correctness on
`normalized` output is proven.

Both modules get a thin PyO3 binding exposing them as Python callables,
so either can be swapped into an existing pipeline without rewriting the
pipeline. `word::normalize` keeps the original's single-word contract
(no internal splitting) — multi-word/sentence input goes through
`sentence::normalize` instead, which is explicitly built for that.

## Phase 2 — sentence-level module (new work, not a port)

This does not exist upstream (confirmed via issue #13/#9 above) — this
is original design, so it needs its own test suite built from real
example sentences with hand-decided expected output, not an oracle
diffed against Python.

### Design

1. Tokenize the input, preserving exact structure (not `str.split()` —
   that loses punctuation/spacing, per the maintainer's own stated
   reason this is hard). Classify each run of characters into one of:
   Bangla word, punctuation, whitespace, emoji, digit, non-Bangla-script
   word, other/unrecognized. Preserve enough position/spacing info to
   reassemble exactly.

2. Auto-detect Bangla per token: check codepoints against the Bangla
   Unicode block (`U+0980`–`U+09FF`, consistent with ranges already
   referenced in `langs.py`). A token is "Bangla" if its word characters
   fall in that range.

3. Routing per token type:
   - Bangla word → `word::normalize`. If it returns `None`, apply the
     configured `NoneTokenPolicy` (below) — do not hardcode one behavior.
   - Emoji → pass through unchanged.
   - Punctuation, whitespace, digits → pass through unchanged.
   - Non-Bangla, non-emoji content (English words, URLs, hashtags,
     mentions) → pass through unchanged by default. Do not pre-build
     per-category configurability (e.g. drop URLs specifically)
     speculatively — add it later only if real corpus inspection shows a
     specific category is an actual problem.

4. Reassemble, preserving original spacing/punctuation placement exactly
   for all pass-through tokens.

### `NoneTokenPolicy` — configurable, not hardcoded

```rust
pub enum NoneTokenPolicy {
    Drop,                  // remove token, close the gap
    KeepOriginal,          // leave original un-normalized text in place
    Placeholder(String),   // replace with a caller-supplied marker
    Error,                 // Result::Err with the offending token's position
    Collect,               // normalize what's possible; return Vec<(position, original)> of failures alongside the result — recommended default for corpus-scale batch processing
}
```

### Step 0.5 (before writing any Phase 2 code) — measure real impact

Run the original Python word-level normalizer over a real, large sample
of the 70GB corpus and report: % of words changed, % that normalize to
`None`, and a spot-check of which words get dropped (garbage/noise vs.
legitimate rare words being lost). This should inform the practical
default policy — don't guess it, measure it.

### Phase 2 test suite

Hand-built, not oracle-diffed (no upstream ground truth exists). Build
from real sentences with emoji, punctuation, mixed Bangla/English,
URLs/hashtags, sourced from social-media-style Bangla data. For each
test sentence, the expected output is hand-decided and hard-coded — this
suite is itself a design artifact worth keeping as evidence that
sentence-level behavior was deliberately specified.

## Build order — Phase 1: core word-level port (do not reorder — each step is a checkpoint)

Phase 2 (sentence module, described above) starts only after Phase 1's
Definition of Done is met below — it depends on a correct
`word::normalize` to route into. Step 0.5 (impact measurement, above)
can run in parallel with Phase 1 any time after Step 0, since it only
needs the original Python library, not the finished Rust port.

### Step 0 — Test harness (build first, before any Rust exists)
Generate a large oracle dataset: run the **original Python library**
over a large, diverse, real sample of the user's actual Bangla corpus
(not synthetic-only data), saving `(input_word, expected_normalized)`
pairs, including the `None` cases. Target 1-5 million word pairs,
stratified across whatever document sources exist in the corpus (OCR'd
text, clean text, social text, etc., if such variety exists). Also
explicitly construct test cases from every docstring example already
present in the Python source (there are several concrete before/after
examples in `normalizer.py`, e.g. the `গ্র্রামকে` → `গ্রামকে` case) to
guarantee every named rule path has at least one direct test.

This oracle file is the single source of truth for "is the Rust port
correct." It must exist and be reviewed before Step 1 begins.

### Step 1 — Data tables
Port all static tables from `langs.py` (Bangla only) into Rust constants
using `once_cell`/`phf`/plain `HashSet`/`HashMap` literals. No logic
yet. Verify by a simple membership-test comparison against the Python
sets (script: for each set/map, assert Rust and Python agree on
contents).

### Step 2 — Word-level ops
Port `mapLegacySymbols`, `fixBrokenDiacritics`, `replaceAssamese`,
`replacePunctuations`. These are simple string-replace passes — lowest
risk, do first, run against oracle subset immediately.

### Step 3 — Core decomp-level ops, one at a time
Port in this order, testing against the oracle after **each individual
function**, not at the end:
1. `fixBrokenNukta`
2. `cleanInvalidUnicodes`
3. `cleanInvalidConnector`
4. `cleanVowelDiacritics` / `cleanConsonantDiacritics` / `fixDiacriticOrder` / `cleanNonCharDiacs` (together = `cleanDiacritics`)
5. `cleanVowelDiacriticComingAfterVowel` (note: Bangla subclass *overrides* the base version — the override adds the `এ` → `ত্র` special case. Make sure the Rust version uses the override's behavior, not the base class's.)
6. `fixNoSpaceChar` (also overridden in the Bangla subclass — same warning)
7. `convertToAndHosonto` + `swapToAndHosontoDiacritics` (the `ত্‍` → `ৎ` ligature logic — this has the deepest lookahead, up to `idx+4`; port very carefully, this is the highest-risk function in the file)
8. `fixTypoForJoFola`, `cleanDoubleCC`, `cleanDoubleRef`, `cleanConnectotForJoFola` (together = `cleanInvalidConjunctDiacritics`, the repeated-fola cleanup)
9. `checkComplexRoot` + `convertComplexRoots` (uses the `conjuncts`/`complex_roots` table — the most "linguistic" logic; test heavily against real conjunct-heavy words)

For each function: port it, run the growing Rust pipeline against the
relevant oracle slice, fix mismatches before moving to the next
function. Do not batch multiple function ports between test runs.

### Step 4 — Assemble the full pipeline
Wire the ops together in the exact order from `Normalizer.__init__`
(word ops dict → decomp-level ops dict, base class entries first, then
Bangla subclass's appended entries) and `baseCompose`'s fixed sequence.
Run against the **full** oracle dataset. Target: 100% exact match. Every
mismatch gets root-caused, not waived.

### Step 5 — Fuzz beyond the oracle
Generate synthetic stress cases: long conjunct chains, nested `্`
sequences, mixed valid/invalid Unicode, empty string, single-char input,
strings that are all diacritics, strings mixing Bangla with the
`allow_english` charset. Compare Rust vs Python on all of these too.

### Step 6 — Performance benchmark
Only after Step 4 and 5 pass at 100%: benchmark Rust vs Python on a
large batch (the kind of volume the actual 70GB corpus implies). Report
words/sec for both, and wall-clock for a fixed-size sample. This number
is what goes in the paper — do not report a speedup estimate before this
is measured.

### Step 7 — PyO3 wrapper + packaging

Primary usage mode is Python — the Rust code is a means to a speed end,
not a goal of switching the pipeline to Rust. End result must be a
normal `pip install`-able package, imported and called like any other
Python function:

```python
import bn_normalize_rs
result = bn_normalize_rs.normalize_word("গ্র্রামকে")   # -> "গ্রামকে" or None
```

**API shape — decide before writing bindings, do not guess:**
Two options, pick one with the user before implementing:
1. **Simplified**: `normalize_word(word: str) -> str | None` — matches
   only the `["normalized"]` value from the original's return dict. No
   `ops` audit trail. Correct default unless the user's existing
   pipeline code specifically depends on the `given`/`ops` fields.
2. **Exact drop-in**: replicate the original's full return shape
   (`{"given": ..., "normalized": ..., "ops": [...]}`) so existing
   pipeline call sites need zero changes. Only worth the extra
   implementation cost if the user confirms their pipeline actually
   reads `ops` or `given` today — check before building this.

Steps:
- Build with `maturin build --release` → produces a `.whl`
- `pip install` the wheel locally, confirm `import bn_normalize_rs`
  works and matches the oracle output through the Python binding layer
  too (not just the Rust-internal tests — the binding itself is a
  place bugs can hide, e.g. string encoding issues at the FFI boundary)
- If publishing: `maturin publish` to PyPI under the same
  `bn-normalize-rs` / `bn_normalize_rs` name used for the crate and repo
- `sentence::normalize` gets the equivalent Python binding once Phase 2
  is implemented, exposing the `NoneTokenPolicy` enum as a Python-side
  parameter (e.g. a string literal or small enum-like class — decide
  based on what's idiomatic for the pipeline calling it)

## Definition of done

- 100% exact match against the oracle dataset (Step 0) and the synthetic
  fuzz set (Step 5) — not 99.9%, not "close enough."
- Every discrepancy found during development is logged in a
  `DIVERGENCES.md` with the specific input, expected output, actual
  output, and root cause — even ones that get fixed. This log is useful
  both for debugging and as evidence for the paper that verification was
  rigorous.
- Measured (not estimated) speedup number from Step 6.
- If the user wants to keep the `allow_english`, `keep_legacy_symbols`,
  `legacy_maps` configuration options from the original constructor,
  confirm which of these are actually used in their pipeline before
  porting all of them — no need to support configuration surface that
  won't be used.

## Explicit non-goals (do not do these unless the user asks)

- Do not port `IndicNormalizer` / other Indic languages (`indic.py`).
- Do not "improve" or "fix" any rule that looks linguistically odd —
  flag it in DIVERGENCES.md and ask, don't silently change behavior.
- Do not optimize for speed at the expense of the mark-then-filter
  semantics described above until correctness is fully proven — get it
  right first, then profile and optimize with the oracle as a guardrail.
- Do not add the `ops` audit-trail feature unless it's needed downstream.