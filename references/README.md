# bn-normalize-rs

A fast Rust reimplementation of Bangla Unicode text normalization, built
for high-throughput corpus processing (e.g. LLM training data pipelines).

Status: **planning / early implementation** — see `plan.md` for the full
build plan.

## Relationship to upstream

This project reimplements the word-level normalization rules from
[bnunicodenormalizer](https://github.com/mnansary/bnUnicodeNormalizer)
(Bengali.AI, MIT licensed) in Rust for performance, and adds a new
sentence-level normalization module that does not exist upstream (see
`THIRD_PARTY_NOTICES.md` for the full attribution and license text, and
`plan.md` for why sentence-level handling is scoped as new work rather
than a port — the upstream maintainer has confirmed in
[issue #13](https://github.com/mnansary/bnUnicodeNormalizer/issues/13)
that this is an open problem, not something we're diverging from).

The word-level core is validated to match the upstream Python library's
output exactly (see `plan.md`, "Definition of done") — it is a faithful,
oracle-tested port, not a reinterpretation.

## Repo layout

```
plan.md                      — full implementation plan (read first)
THIRD_PARTY_NOTICES.md        — upstream license + attribution
reference_python_source/      — pinned snapshot of upstream v0.1.7, used
                                 only to generate the correctness oracle
DIVERGENCES.md                — log of any behavioral discrepancies
                                 found during porting (created during
                                 implementation)
src/word/                     — faithful port of upstream normalization
src/sentence/                 — new sentence-level module (original work)
```

## Why this exists

Word-level Bangla Unicode normalization is essential for LLM training
data quality — visually-identical Bangla text can be encoded as
different Unicode codepoint sequences (e.g. the vowel sign ো as one
codepoint vs. two combining codepoints), which fragments tokenization
and dilutes training signal. The upstream Python implementation is
correct but slow (pure Python, word-at-a-time), which becomes a
bottleneck at large-corpus scale. This project provides a
behaviorally-identical, much faster implementation.

## License

MIT — see `LICENSE`. See `THIRD_PARTY_NOTICES.md` for upstream
attribution required by the upstream library's own MIT license.
