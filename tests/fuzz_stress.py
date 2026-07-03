#!/usr/bin/env python3
"""
Step 5 — Fuzz beyond the oracle.

Generates synthetic stress cases and cross-validates Rust (via Python bindings)
against the upstream Python library. This catches edge cases that the real
corpus oracle might miss.

Categories:
  1. Long conjunct chains
  2. Nested hosonto (্) sequences
  3. Mixed valid/invalid Unicode
  4. Empty string / single-char input
  5. All-diacritics strings
  6. Bangla mixed with allow_english charset
  7. Random Bangla codepoint sequences
  8. Boundary conditions (all vowels, all consonants, etc.)
"""

import random
import sys
import json
from datetime import datetime

try:
    import bn_normalize_rs
except ImportError:
    print("ERROR: bn_normalize_rs not installed. Run: maturin develop --release")
    sys.exit(1)

from bnunicodenormalizer import Normalizer

# Bangla Unicode ranges
BANGLA_START = 0x0980
BANGLA_END = 0x09FF

# Key Bangla characters
VOWELS = list("অআইঈউঊঋএঐওঔ")
CONSONANTS = list("কখগঘঙচছজঝঞটঠডঢণতথদধনপফবভমযরলশষসহ")
VOWEL_DIACRITICS = list("ািীুূৃেৈোৌ")
CONSONANT_DIACRITICS = list("ঁংঃ")
CONNECTOR = '্'  # hosonto U+09CD
NUKTA = '়'      # U+09BC
SPECIAL = ['ৎ', 'ড়', 'ঢ়', 'য়']
ZWJ = '\u200D'
ZWNJ = '\u200C'

random.seed(42)  # reproducible


def random_bangla_chars(n):
    """Generate n random chars from the Bangla Unicode block."""
    return ''.join(chr(random.randint(BANGLA_START, BANGLA_END)) for _ in range(n))


def generate_stress_cases():
    """Generate all categories of synthetic stress inputs."""
    cases = []

    # ── Category 1: Long conjunct chains ──
    for length in [3, 5, 8, 12, 20]:
        # Valid-ish conjunct: consonant + hosonto + consonant + ...
        word = ''
        for i in range(length):
            word += random.choice(CONSONANTS)
            if i < length - 1:
                word += CONNECTOR
        cases.append(('long_conjunct', word))

    # ── Category 2: Nested hosonto sequences ──
    cases.append(('nested_hosonto', 'ক্্্্খ'))
    cases.append(('nested_hosonto', 'গ্্ঘ্্ঙ'))
    cases.append(('nested_hosonto', CONNECTOR * 5))
    cases.append(('nested_hosonto', 'ক' + CONNECTOR * 10 + 'খ'))
    cases.append(('nested_hosonto', CONNECTOR + 'ক' + CONNECTOR))

    # ── Category 3: Mixed valid/invalid Unicode ──
    cases.append(('mixed_unicode', 'ক\x00খ'))       # null byte
    cases.append(('mixed_unicode', 'কখ\ufffd'))     # replacement char
    cases.append(('mixed_unicode', 'ক\u200Bখ'))     # zero-width space
    cases.append(('mixed_unicode', 'ক\u00A0খ'))     # non-breaking space
    cases.append(('mixed_unicode', 'ক\u2028খ'))     # line separator
    cases.append(('mixed_unicode', 'আ\u0300মি'))     # combining grave accent
    cases.append(('mixed_unicode', '\ufeffকখ'))     # BOM
    for _ in range(10):
        word = random_bangla_chars(random.randint(1, 15))
        cases.append(('mixed_unicode', word))

    # ── Category 4: Empty / single-char / minimal ──
    cases.append(('minimal', ''))
    cases.append(('minimal', ' '))
    cases.append(('minimal', '\t'))
    cases.append(('minimal', '\n'))
    for c in VOWELS + CONSONANTS + VOWEL_DIACRITICS + CONSONANT_DIACRITICS:
        cases.append(('single_char', c))
    cases.append(('single_char', CONNECTOR))
    cases.append(('single_char', NUKTA))
    cases.append(('single_char', ZWJ))
    cases.append(('single_char', ZWNJ))
    for c in SPECIAL:
        cases.append(('single_char', c))

    # ── Category 5: All-diacritics strings ──
    for length in [1, 2, 5, 10]:
        word = ''.join(random.choice(VOWEL_DIACRITICS) for _ in range(length))
        cases.append(('all_vowel_diacs', word))
        word = ''.join(random.choice(CONSONANT_DIACRITICS) for _ in range(length))
        cases.append(('all_consonant_diacs', word))
        word = ''.join(random.choice(VOWEL_DIACRITICS + CONSONANT_DIACRITICS) for _ in range(length))
        cases.append(('all_diacritics', word))

    # ── Category 6: Bangla mixed with English ──
    cases.append(('bangla_english', 'হ্যালোWorld'))
    cases.append(('bangla_english', 'ABCকখগDEF'))
    cases.append(('bangla_english', '123বাংলা456'))
    cases.append(('bangla_english', 'a' * 100))
    cases.append(('bangla_english', 'test@email.com'))
    cases.append(('bangla_english', '#হ্যাশট্যাগ'))
    cases.append(('bangla_english', 'https://bn.wikipedia.org'))

    # ── Category 7: Random Bangla codepoint sequences ──
    for _ in range(200):
        length = random.randint(1, 30)
        word = random_bangla_chars(length)
        cases.append(('random_bangla', word))

    # ── Category 8: Boundary conditions ──
    # All vowels concatenated
    cases.append(('boundary', ''.join(VOWELS)))
    # All consonants concatenated
    cases.append(('boundary', ''.join(CONSONANTS)))
    # Consonant + every vowel diacritic
    for vd in VOWEL_DIACRITICS:
        cases.append(('boundary', 'ক' + vd))
    # Every consonant + hosonto + য (jo-fola)
    for c in CONSONANTS:
        cases.append(('boundary', c + CONNECTOR + 'য'))
    # Every consonant + hosonto + র (ref)
    for c in CONSONANTS:
        cases.append(('boundary', c + CONNECTOR + 'র'))
    # ত + hosonto + various chars (to+hosonto edge)
    for c in CONSONANTS + VOWELS:
        cases.append(('to_hosonto', 'ত' + CONNECTOR + c))
    # Repeated same consonant chains
    for c in ['ক', 'ত', 'স', 'ন']:
        for reps in [2, 3, 5]:
            word = (c + CONNECTOR) * reps + c
            cases.append(('repeated', word))
    # ZWJ/ZWNJ patterns
    cases.append(('zwj', 'র' + ZWJ))
    cases.append(('zwj', ZWJ + 'ক'))
    cases.append(('zwj', 'ক' + CONNECTOR + ZWJ + 'য'))
    cases.append(('zwj', 'র' + ZWJ + CONNECTOR + 'য'))
    cases.append(('zwnj', ZWNJ + 'ক'))
    cases.append(('zwnj', 'ক' + ZWNJ + 'খ'))

    # ── Category 9: Nukta patterns ──
    cases.append(('nukta', 'ড' + NUKTA))        # ড + nukta → ড়
    cases.append(('nukta', 'ঢ' + NUKTA))        # ঢ + nukta → ঢ়
    cases.append(('nukta', 'য' + NUKTA))        # য + nukta → য়
    cases.append(('nukta', 'ক' + NUKTA))        # invalid nukta base
    cases.append(('nukta', NUKTA + 'ক'))        # leading nukta
    cases.append(('nukta', NUKTA * 5))          # multiple nuktas
    cases.append(('nukta', 'ড' + NUKTA + NUKTA))  # double nukta

    # ── Category 10: Extreme lengths ──
    cases.append(('extreme_len', 'ক' * 500))
    cases.append(('extreme_len', ('ক' + CONNECTOR) * 100 + 'ক'))
    cases.append(('extreme_len', ''.join(random.choice(VOWEL_DIACRITICS) for _ in range(200))))

    return cases


def main():
    normalizer = Normalizer()
    cases = generate_stress_cases()

    print(f"Fuzz stress test — {len(cases)} synthetic cases")
    print(f"Timestamp: {datetime.now().isoformat()}")
    print()

    total = 0
    matched = 0
    mismatches = []
    panics = []

    for category, word in cases:
        total += 1

        # Python upstream
        try:
            py_result = normalizer(word)
            py_normalized = py_result.get('normalized')
        except Exception as e:
            py_normalized = f"PYTHON_ERROR:{e}"

        # Rust via Python bindings
        try:
            rs_normalized = bn_normalize_rs.normalize_word(word)
        except Exception as e:
            rs_normalized = f"RUST_ERROR:{e}"
            panics.append((category, word, str(e)))

        if py_normalized == rs_normalized:
            matched += 1
        else:
            mismatches.append({
                'category': category,
                'input': word,
                'input_codepoints': [f'U+{ord(c):04X}' for c in word],
                'python': py_normalized,
                'rust': rs_normalized,
            })

    # Also test with allow_english=True
    english_cases = [
        ('ASD123', True),
        ('hello', True),
        ('test@email.com', True),
        ('বাংলাEnglish', True),
        ('', True),
        ('123', True),
    ]

    for word, allow_eng in english_cases:
        total += 1
        try:
            py_result = Normalizer(allow_english=allow_eng)(word)
            py_normalized = py_result.get('normalized')
        except Exception as e:
            py_normalized = f"PYTHON_ERROR:{e}"

        try:
            rs_normalized = bn_normalize_rs.normalize_word_with_options(
                word, allow_english=allow_eng
            )
        except Exception as e:
            rs_normalized = f"RUST_ERROR:{e}"
            panics.append(('allow_english', word, str(e)))

        if py_normalized == rs_normalized:
            matched += 1
        else:
            mismatches.append({
                'category': 'allow_english',
                'input': word,
                'input_codepoints': [f'U+{ord(c):04X}' for c in word],
                'python': py_normalized,
                'rust': rs_normalized,
            })

    print(f"Results: {matched}/{total} matched")
    print(f"Mismatches: {len(mismatches)}")
    print(f"Panics/errors: {len(panics)}")
    print()

    if mismatches:
        print("=== MISMATCHES ===")
        for m in mismatches[:50]:
            print(f"  [{m['category']}] input={m['input']!r}")
            print(f"    codepoints: {m['input_codepoints']}")
            print(f"    python:     {m['python']!r}")
            print(f"    rust:       {m['rust']!r}")
            print()

    if panics:
        print("=== PANICS/ERRORS ===")
        for cat, word, err in panics:
            print(f"  [{cat}] {word!r}: {err}")

    # Save full results
    results = {
        'timestamp': datetime.now().isoformat(),
        'total': total,
        'matched': matched,
        'mismatches': mismatches,
        'panics': [{'category': c, 'input': w, 'error': e} for c, w, e in panics],
    }
    with open('tests/fuzz_results.json', 'w') as f:
        json.dump(results, f, ensure_ascii=False, indent=2)

    if mismatches or panics:
        print(f"\nFAILED: {len(mismatches)} mismatches, {len(panics)} panics")
        sys.exit(1)
    else:
        print("ALL PASSED — 100% match on synthetic stress cases ✓")


if __name__ == '__main__':
    main()
