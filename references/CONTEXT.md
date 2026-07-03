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

## What needs to be done next

| Step | What | Priority |
|---|---|---|
| Step 5 | Fuzz testing (synthetic stress cases) | Medium |
| Step 6 | Performance benchmark (Rust vs Python words/sec) | Medium |
| Step 7 | PyO3 bindings → `pip install`-able Python package | High |
| Phase 2 | Sentence-level module (tokenize, NoneTokenPolicy) | Medium |

## Architecture notes

### Key design decisions

- **Pre-composed Bangla chars** — ড় (U+09DC), ঢ় (U+09DD), য় (U+09DF) are stored as pre-composed single-codepoint forms everywhere: nukta_map, consonants, conjuncts, complex_roots.

- **`Vec<Option<String>>` decomp** — mark-then-filter pattern. Each slot is `Option<String>`.

- **`safeop` wrapper** — rejoin → resplit before/after each op. Pre-composed chars (U+09DF etc.) survive resplit as single chars.

- **`fixTypoForJoFola`** — compares against U+09DF to convert য় back to য after hosonto (matching installed library behavior).

### Reference Python source

The upstream Python source is in `references/`. **The installed library is the ground truth** — verify against `pip` version, not reference copies.
