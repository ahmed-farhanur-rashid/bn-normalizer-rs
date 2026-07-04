# DIVERGENCES.md

> Log of every discrepancy found between the Rust port and the upstream
> Python library during development. All entries were root-caused and
> fixed. This log serves as evidence of rigorous verification per the
> project's Definition of Done.

## Summary

- **Total discrepancies found**: 6
- **All resolved**: ✅ Yes — 100% oracle match (50,008 words) + 100% fuzz match (467 synthetic cases)
- **Root cause pattern**: 5 of 6 were caused by the reference Python *source files* containing decomposed multi-codepoint Bangla characters, while the *installed pip package* uses pre-composed single-codepoint forms

## Critical lesson

> The reference Python source files in `bnUnicodeNormalizer-src/` contain decomposed
> multi-codepoint Bangla characters (base + nukta). The **installed pip
> package** has different codepoints in several places (pre-composed
> single-codepoint forms). **Always verify against the installed library,
> not the reference source.**

---

## Divergence 1: Nukta map codepoints

| | |
|---|---|
| **Discovered** | During oracle validation (Step 4) |
| **Symptom** | Nukta normalization produced wrong output for ড+়, ঢ+়, য+় |
| **Root cause** | Python's installed `nukta_map` maps to pre-composed single-codepoint forms: U+09DC (ড়), U+09DD (ঢ়), U+09DF (য়). Our Rust source literals used decomposed 2-codepoint forms (base char + U+09BC nukta). |
| **Fix** | Changed nukta map values to use pre-composed escape sequences: `\u{09DC}`, `\u{09DD}`, `\u{09DF}` |
| **Status** | ✅ Fixed |

## Divergence 2: Curly quote in punctuation sets

| | |
|---|---|
| **Discovered** | During oracle validation (Step 4) |
| **Symptom** | Punctuation replacement behaved differently for curly quotes |
| **Root cause** | Python's punctuations set has `"` (U+201D, RIGHT DOUBLE QUOTATION MARK / closing), not `"` (U+201C, LEFT DOUBLE QUOTATION MARK / opening). Our Rust tables had the wrong one. |
| **Fix** | Fixed all 4 punctuation-related sets to use U+201D |
| **Status** | ✅ Fixed |

## Divergence 3: Nukta in VALID_CHARS

| | |
|---|---|
| **Discovered** | During oracle validation (Step 4) |
| **Symptom** | Words with standalone nukta (U+09BC) were not being cleaned |
| **Root cause** | Python does NOT include standalone nukta (U+09BC) in its valid character set. Our Rust `VALID_CHARS` incorrectly included it. |
| **Fix** | Removed U+09BC from `VALID_CHARS` |
| **Status** | ✅ Fixed |

## Divergence 4: Pre-composed consonants missing

| | |
|---|---|
| **Discovered** | During oracle validation (Step 4) |
| **Symptom** | ড় (U+09DC), ঢ় (U+09DD), য় (U+09DF) were being stripped as invalid |
| **Root cause** | These pre-composed consonant forms were not in `CONSONANTS_SINGLE`, so they weren't in `VALID_CHARS` or `CONSONANTS_SINGLE_SET`. |
| **Fix** | Added U+09DC, U+09DD, U+09DF to `CONSONANTS_SINGLE` |
| **Status** | ✅ Fixed |

## Divergence 5: Curly quote punctuation map

| | |
|---|---|
| **Discovered** | During oracle validation (Step 4) |
| **Symptom** | Smart/curly single quotes were being mapped to straight quotes |
| **Root cause** | The installed Python library does NOT map `'` (U+2018) / `'` (U+2019) to straight quotes in its punctuation map. Our Rust table had those entries. |
| **Fix** | Removed the `'`→`'` and `'`→`'` entries from `PUNCTUATION_MAP` |
| **Status** | ✅ Fixed |

## Divergence 6: fixTypoForJoFola codepoint mismatch

| | |
|---|---|
| **Discovered** | During oracle validation (Step 4) |
| **Symptom** | Jo-fola typo correction didn't fire for certain conjunct patterns |
| **Root cause** | The installed library compares against U+09DF (pre-composed য়, 1 codepoint) in `fixTypoForJoFola`, not the decomposed `য়` (য + ়, 2 codepoints) from the reference source. Also, the conjuncts table entry for `য়্য` needed the pre-composed form. |
| **Fix** | Changed comparison to `\u{09DF}` and fixed the conjuncts table entry |
| **Status** | ✅ Fixed |

---

## Verification

After all fixes:

| Test Suite | Result |
|---|---|
| Oracle (50,008 real corpus words) | 50,008/50,008 (100%) |
| Synthetic fuzz (467 stress cases) | 467/467 (100%) |
| Python binding oracle validation | 50,008/50,008 (100%) |
| Unit tests | 31/31 |
| Doc tests | 2/2 |
