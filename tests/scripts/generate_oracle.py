#!/usr/bin/env python3
"""
Generate a larger oracle by creating diverse synthetic + real-pattern Bangla words.

This supplements the built-in 8 cases with hundreds of words that exercise
every normalization rule path, without needing an external corpus file.
"""
import json
import sys

try:
    from bnunicodenormalizer import Normalizer
except ImportError:
    print("ERROR: bnunicodenormalizer not installed")
    sys.exit(1)

# ── Synthetic test words covering every rule path ──

WORDS = [
    # Already clean — should pass through unchanged
    "বাংলাদেশ", "কম্পিউটার", "প্রযুক্তি", "বিশ্ববিদ্যালয়", "সরকার",
    "মানুষ", "পরিবেশ", "উন্নয়ন", "শিক্ষা", "সংস্কৃতি",
    "রাজনীতি", "অর্থনীতি", "প্রকৌশল", "চিকিৎসা", "গবেষণা",
    "ব্যবস্থাপনা", "তথ্যপ্রযুক্তি", "যোগাযোগ", "পরিচালনা", "নিরাপত্তা",

    # Broken diacritics (ে + া → ো, ে + ৗ → ৌ)
    "আরো", "পৌঁছে", "সংস্কৄতি",

    # Nukta cases (য+় → য়, ব+় → র, ড+় → ড়, ঢ+় → ঢ়)
    "কেন্দ্রীয়", "রযে়ছে", "জ়ন্য",

    # Invalid hosonto
    "দুই্টি", "এ্তে", "নেট্ওয়ার্ক", "এস্আই", "চু্ক্তি", "যু্ক্ত", "কিছু্ই",

    # To+hosonto
    "বুত্পত্তি", "উত্স", "বৎসর", "উৎসাহ", "মাৎস্য",

    # Double/duplicate diacritics
    "যুুদ্ধ", "দুুই", "প্রকৃৃতির", "আমাকোা",

    # Vowel + vowel diacritic
    "উুলু", "আর্কিওোলজি", "একএে",

    # Repeated fola
    "গ্র্রামকে",

    # Invalid start
    "াটোবাকো", "িত", "্কর",

    # Ending hosonto
    "অজানা্",

    # Invalid connector / broken connector
    "সং্যুক্তি",

    # Non-Bangla (should return None)
    "ASD123", "hello", "12345", "@#$%",

    # Single chars
    "অ", "আ", "ক", "।", "০", "ৎ",

    # Legacy symbols
    "৺", "৻", "ঀ", "ঌ", "ৡ", "ঽ", "ৠ", "৲", "৴", "৵", "৶", "৷", "৸", "৹",

    # Broken diacritic edge case
    "অা", "ৄ",

    # Complex conjuncts
    "ক্ষমা", "জ্ঞান", "ঞ্চল", "ক্ষুধা", "জ্বর",
    "বিদ্যুৎ", "শক্তি", "যুক্ত", "মুক্তি", "ব্যক্তি",
    "নিষ্ক্রিয়", "বিশ্লেষণ", "সংক্ষিপ্ত", "দক্ষিণ", "পূর্ব",

    # ZWNJ/ZWJ edge cases
    "\u200Cটেস্ট", "\u200Dটেস্ট",

    # Assamese chars
    "ৰাম", "ৱাটার",

    # Punctuation normalization
    "\u201Cটেস্ট\u201D", "৷৷",

    # Mixed valid patterns
    "আন্তর্জাতিক", "পারমাণবিক", "গণতান্ত্রিক", "ঐতিহাসিক", "ভৌগোলিক",
    "সাংস্কৃতিক", "প্রাতিষ্ঠানিক", "বৈজ্ঞানিক", "রাষ্ট্রবিজ্ঞান",

    # Edge: empty-ish
    "", " ",

    # Connector-heavy
    "স্ক্র্যাচ", "স্ট্র্যাটেজি",

    # Real words that commonly appear in Bangla corpora
    "করেছে", "হয়েছে", "বলেছেন", "জানিয়েছেন", "দেখা", "থাকে", "হবে",
    "করতে", "বলতে", "দিতে", "নিতে", "আসতে", "যেতে", "খেতে",
    "পড়াশোনা", "ছেলেমেয়ে", "ভালোবাসা", "জানালা", "আকাশ",
]

def main():
    bnorm = Normalizer()
    results = []
    errors = []

    for word in WORDS:
        if not word.strip() or " " in word.strip():
            # Skip empty or multi-word (normalizer raises on multi-word)
            continue
        try:
            result = bnorm(word)
            normalized = result.get("normalized") if result else None
        except Exception as e:
            normalized = None
            errors.append(f"  EXCEPTION on {word!r}: {e}")
        results.append({"input": word, "expected_normalized": normalized})

    outpath = "tests/oracle.jsonl"
    with open(outpath, "w", encoding="utf-8") as f:
        for r in results:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")

    total = len(results)
    dropped = sum(1 for r in results if r["expected_normalized"] is None)
    changed = sum(1 for r in results if r["expected_normalized"] is not None and r["expected_normalized"] != r["input"])
    unchanged = total - dropped - changed

    print(f"Oracle generated: {outpath}")
    print(f"  Total: {total}  Unchanged: {unchanged}  Changed: {changed}  Dropped: {dropped}")
    if errors:
        print("Errors:")
        for e in errors:
            print(e)

if __name__ == "__main__":
    main()
