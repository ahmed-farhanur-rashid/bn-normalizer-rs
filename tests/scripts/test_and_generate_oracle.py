#!/usr/bin/env python3
"""
One-shot test/oracle-generation script for bnunicodenormalizer.

Usage:
    python3 test_and_generate_oracle.py                     # built-in test cases only
    python3 test_and_generate_oracle.py corpus.txt           # + sample from a text file
    python3 test_and_generate_oracle.py corpus.txt 5000      # + custom word sample size

Outputs:
    oracle.jsonl        - (input, expected_normalized) pairs, one JSON object per line
    oracle_summary.txt  - human-readable stats: % changed, % dropped to None, samples

This script does NOT touch Rust. It's purely for (a) sanity-checking that
the original library behaves the way plan.md describes, and (b) producing
a first oracle file to validate the Rust port against later.
"""
import sys
import json
import random
from collections import Counter

try:
    from bnunicodenormalizer import Normalizer
except ImportError:
    print("ERROR: bnunicodenormalizer not installed. Run:")
    print("  pip install bnunicodenormalizer --break-system-packages")
    sys.exit(1)


# ---------------------------------------------------------------------------
# Known tricky cases, pulled directly from the library's own docstrings.
# These pin down the exact rule-paths plan.md calls out as highest-risk.
# ---------------------------------------------------------------------------
BUILTIN_CASES = [
    # (input, note)
    ("গ্র্রামকে", "repeated fola / double-র্র cleanup (cleanDoubleRef)"),
    ("যুুদ্ধ", "duplicate vowel diacritic"),
    ("উুলু", "vowel diacritic after vowel"),
    ("আর্কিওোলজি", "vowel diacritic after vowel (ও + ো)"),
    ("একএে", "এ + vowel-diacritic -> ত্র normalization special case"),
    ("াটোবাকো", "leading invalid unicode (vowel diacritic with no base)"),
    ("অা", "broken diacritic: অ + া should become আ"),
    ("ৄ", "broken diacritic: should normalize to ৃ"),
]


def run_word(bnorm, word):
    """Run one word through the normalizer, return (normalized_or_None, ops)."""
    try:
        result = bnorm(word)
    except Exception as e:
        return None, [{"operation": "EXCEPTION", "before": word, "after": str(e)}]
    if result is None:
        return None, []
    return result.get("normalized"), result.get("ops", [])


def print_builtin_cases(bnorm):
    print("=" * 70)
    print("BUILT-IN DOCSTRING TEST CASES")
    print("=" * 70)
    for word, note in BUILTIN_CASES:
        normalized, ops = run_word(bnorm, word)
        status = "OK" if normalized is not None else "DROPPED TO NONE"
        print(f"\n  input:      {word}")
        print(f"  normalized: {normalized}   [{status}]")
        print(f"  note:       {note}")
        if ops:
            for op in ops:
                print(f"    - {op['operation']}: {op['before']!r} -> {op['after']!r}")
    print()


def sample_words_from_file(path, n):
    """Extract up to n unique whitespace-split tokens from a text file."""
    words = set()
    with open(path, "r", encoding="utf-8", errors="ignore") as f:
        for line in f:
            for tok in line.strip().split():
                # crude cleanup: strip common punctuation stuck to word edges
                tok = tok.strip("।,.!?\"'()[]{}:;-–—")
                if tok:
                    words.add(tok)
            if len(words) >= n * 3:  # overcollect, then sample down
                break
    words = list(words)
    random.shuffle(words)
    return words[:n]


def main():
    bnorm = Normalizer()

    print_builtin_cases(bnorm)

    all_words = [w for w, _ in BUILTIN_CASES]

    if len(sys.argv) > 1:
        corpus_path = sys.argv[1]
        n = int(sys.argv[2]) if len(sys.argv) > 2 else 2000
        print(f"Sampling up to {n} words from {corpus_path} ...")
        try:
            sampled = sample_words_from_file(corpus_path, n)
            print(f"  collected {len(sampled)} unique words")
            all_words.extend(sampled)
        except FileNotFoundError:
            print(f"  WARNING: {corpus_path} not found, skipping corpus sample")
    else:
        print("No corpus file given — oracle will only contain the built-in cases.")
        print("Usage: python3 test_and_generate_oracle.py <path_to_text_file> [sample_size]")

    print(f"\nRunning normalizer over {len(all_words)} total words ...")

    results = []
    dropped = 0
    changed = 0
    unchanged = 0
    op_counter = Counter()

    for word in all_words:
        normalized, ops = run_word(bnorm, word)
        results.append({"input": word, "expected_normalized": normalized})
        if normalized is None:
            dropped += 1
        elif normalized != word:
            changed += 1
            for op in ops:
                op_counter[op["operation"]] += 1
        else:
            unchanged += 1

    with open("oracle.jsonl", "w", encoding="utf-8") as f:
        for r in results:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")

    total = len(all_words)
    summary_lines = [
        "ORACLE GENERATION SUMMARY",
        "=" * 70,
        f"Total words processed:   {total}",
        f"Unchanged:                {unchanged} ({100*unchanged/total:.2f}%)",
        f"Changed by normalization: {changed} ({100*changed/total:.2f}%)",
        f"Dropped to None:          {dropped} ({100*dropped/total:.2f}%)",
        "",
        "Operations triggered (across all changed words):",
    ]
    for op_name, count in op_counter.most_common():
        summary_lines.append(f"  {op_name}: {count}")

    if dropped > 0:
        summary_lines.append("")
        summary_lines.append(f"Sample of dropped words (up to 20):")
        drop_samples = [r["input"] for r in results if r["expected_normalized"] is None][:20]
        for w in drop_samples:
            summary_lines.append(f"  {w}")

    summary_text = "\n".join(summary_lines)
    print("\n" + summary_text)

    with open("oracle_summary.txt", "w", encoding="utf-8") as f:
        f.write(summary_text + "\n")

    print(f"\nWrote oracle.jsonl ({total} entries) and oracle_summary.txt")
    print("Next: point the Rust test harness at oracle.jsonl and diff its output.")


if __name__ == "__main__":
    main()
