//! Static data tables for Bangla, ported from `langs.py`.

use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

// ─── Character constants ───────────────────────────────────────────────
pub const NUKTA: char = '\u{09BC}';
pub const CONNECTOR: char = '\u{09CD}'; // ্ (hosonto)
pub const KHANDOTA: char = 'ৎ';

// ─── Core character lists ──────────────────────────────────────────────

pub static VOWELS: &[char] = &[
    'অ', 'আ', 'ই', 'ঈ', 'উ', 'ঊ', 'ঋ', 'এ', 'ঐ', 'ও', 'ঔ',
];

pub static VOWELS_SET: Lazy<HashSet<char>> = Lazy::new(|| VOWELS.iter().copied().collect());

/// Single-codepoint consonants only (no ড়, ঢ়, য় which are base+nukta).
pub static CONSONANTS_SINGLE: &[char] = &[
    'ক', 'খ', 'গ', 'ঘ', 'ঙ', 'চ', 'ছ', 'জ', 'ঝ', 'ঞ',
    'ট', 'ঠ', 'ড', 'ঢ', 'ণ', 'ত', 'থ', 'দ', 'ধ', 'ন',
    'প', 'ফ', 'ব', 'ভ', 'ম', 'য', 'র', 'ল', 'শ', 'ষ',
    'স', 'হ', 'ৎ',
    '\u{09DC}', '\u{09DD}', '\u{09DF}', // ড়, ঢ়, য় (pre-composed)
];

pub static CONSONANTS_SINGLE_SET: Lazy<HashSet<char>> = Lazy::new(|| {
    CONSONANTS_SINGLE.iter().copied().collect()
});

pub static VOWEL_DIACRITICS: &[char] = &['া', 'ি', 'ী', 'ু', 'ূ', 'ৃ', 'ে', 'ৈ', 'ো', 'ৌ'];
pub static VOWEL_DIACRITICS_SET: Lazy<HashSet<char>> = Lazy::new(|| VOWEL_DIACRITICS.iter().copied().collect());

pub static CONSONANT_DIACRITICS: &[char] = &['ঁ', 'ং', 'ঃ'];
pub static CONSONANT_DIACRITICS_SET: Lazy<HashSet<char>> = Lazy::new(|| CONSONANT_DIACRITICS.iter().copied().collect());

pub static DIACRITICS: Lazy<HashSet<char>> = Lazy::new(|| {
    let mut s: HashSet<char> = VOWEL_DIACRITICS.iter().copied().collect();
    s.extend(CONSONANT_DIACRITICS.iter());
    s
});

pub static NUMBERS: &[char] = &['০', '১', '২', '৩', '৪', '৫', '৬', '৭', '৮', '৯'];

pub static NON_GLYPH_UNICODES: &[char] = &[
    '\u{0984}', '\u{098D}', '\u{098E}', '\u{0991}', '\u{0992}',
    '\u{09A9}', '\u{09B1}', '\u{09B3}', '\u{09B4}', '\u{09B5}',
    '\u{09BA}', '\u{09BB}', '\u{09C5}', '\u{09C6}', '\u{09C9}',
    '\u{09CA}', '\u{09CF}', '\u{09D0}', '\u{09D1}', '\u{09D2}',
    '\u{09D3}', '\u{09D4}', '\u{09D5}', '\u{09D6}', '\u{09D8}',
    '\u{09D9}', '\u{09DA}', '\u{09DB}', '\u{09DE}', '\u{09E4}',
    '\u{09E5}', 'ৼ', '৽', '৾', '\u{09FF}',
];

pub static LEGACY_SYMBOLS: &[char] = &['৺', '৻', 'ঀ', 'ঌ', 'ৡ', 'ঽ', 'ৠ', '৲', '৴', '৵', '৶', '৷', '৸', '৹'];
pub static LEGACY_SYMBOLS_SET: Lazy<HashSet<char>> = Lazy::new(|| LEGACY_SYMBOLS.iter().copied().collect());

pub static DEFAULT_LEGACY_MAPS: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ('ঀ', "৭"), ('ঌ', "৯"), ('ৡ', "৯"), ('৵', "৯"),
        ('৻', "ৎ"), ('ৠ', "ঋ"), ('ঽ', "ই"),
    ])
});

// ─── Composed sets ─────────────────────────────────────────────────────

pub static NON_CHARS: Lazy<HashSet<char>> = Lazy::new(|| {
    let mut s = HashSet::new();
    s.extend(NUMBERS.iter());
    for &p in &['!', '"', '\'', '(', ')', ',', '-', '.', ':', ';', '<', '=', '>', '?', '[', ']', '{', '}', '।', '৷', '–', '—', '\u{201D}', '√'] {
        s.insert(p);
    }
    s.insert('৳');
    s.extend(NON_GLYPH_UNICODES.iter());
    s.extend(LEGACY_SYMBOLS.iter());
    s
});

pub static VALID_CHARS: Lazy<HashSet<char>> = Lazy::new(|| {
    let mut s = HashSet::new();
    s.insert(' ');
    s.extend(VOWELS.iter());
    s.extend(CONSONANTS_SINGLE.iter());
    s.extend(VOWEL_DIACRITICS.iter());
    s.extend(CONSONANT_DIACRITICS.iter());
    s.extend(NUMBERS.iter());
    for &p in &['!', '"', '\'', '(', ')', ',', '-', '.', ':', ';', '<', '=', '>', '?', '[', ']', '{', '}', '।', '৷', '–', '—', '\u{201D}', '√'] {
        s.insert(p);
    }
    s.insert(CONNECTOR);
    s.insert('\u{200D}');
    s.insert('\u{200C}');
    s
});

pub static INVALID_STARTS: Lazy<HashSet<char>> = Lazy::new(|| {
    let mut s: HashSet<char> = DIACRITICS.iter().copied().collect();
    s.insert(CONNECTOR);
    s
});

pub static INVALID_CONNECTORS: Lazy<HashSet<char>> = Lazy::new(|| {
    let mut s: HashSet<char> = INVALID_STARTS.iter().copied().collect();
    s.extend(VOWELS.iter());
    s.insert(KHANDOTA);
    s.extend(NUMBERS.iter());
    for &p in &['!', '"', '\'', '(', ')', ',', '-', '.', ':', ';', '<', '=', '>', '?', '[', ']', '{', '}', '।', '৷', '–', '—', '\u{201D}', '√'] {
        s.insert(p);
    }
    s
});

pub static ENGLISH_VALID: Lazy<HashSet<char>> = Lazy::new(|| {
    let mut s = HashSet::new();
    for c in 'a'..='z' { s.insert(c); }
    for c in 'A'..='Z' { s.insert(c); }
    for c in '0'..='9' { s.insert(c); }
    for &c in &['!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
               ':', ';', '<', '=', '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~'] {
        s.insert(c);
    }
    s
});

pub static VALID_AFTER_TO_HOSONTO: Lazy<HashSet<char>> = Lazy::new(|| {
    ['ত', 'থ', 'ন', 'ব', 'ম', 'য', 'র'].into_iter().collect()
});

// ─── Normalization maps ────────────────────────────────────────────────

pub static NUKTA_MAP_STR: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    // Pre-composed single-codepoint forms, matching Python's output.
    let mut m = HashMap::new();
    m.insert('\u{09AF}', "\u{09DF}"); // য → য় (U+09DF)
    m.insert('\u{09AC}', "\u{09B0}"); // ব → র
    m.insert('\u{09A1}', "\u{09DC}"); // ড → ড় (U+09DC)
    m.insert('\u{09A2}', "\u{09DD}"); // ঢ → ঢ় (U+09DD)
    m
});

pub static DIACRITIC_MAP: &[(&str, &str)] = &[
    ("ো", "ো"), ("ৌ", "ৌ"), ("অা", "আ"), ("ৄ", "ৃ"),
];

pub static ASSAMESE_MAP: &[(&str, &str)] = &[("ৰ", "র"), ("ৱ", "ব")];

pub static PUNCTUATION_MAP: &[(&str, &str)] = &[
    ("৷", "।"), ("–", "-"),
];

// ─── Conjuncts ─────────────────────────────────────────────────────────

pub static CONJUNCTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "এ্য","অ্য","ক্ক","ক্ট","ক্ট্য","ক্ট্র","ক্ত","ক্ত্র","ক্ব","ক্ম","ক্য","ক্র","ক্র্য","ক্ল","ক্ল্য","ক্ষ","ক্ষ্ণ","ক্ষ্ব","ক্ষ্ম","ক্ষ্ম্য","ক্ষ্য",
        "ক্স","ক্স্য","খ্য","খ্র","গ্ণ","গ্ধ","গ্ধ্য","গ্ধ্র","গ্ন","গ্ন্য","গ্ব","গ্ম","গ্য","গ্র","গ্র্য","গ্ল","গ্ল্য","ঘ্ন","ঘ্য","ঘ্র",
        "ঙ্ক","ঙ্ক্ত","ঙ্ক্য","ঙ্ক্ষ","ঙ্খ","ঙ্খ্য","ঙ্গ","ঙ্গ্য","ঙ্ঘ","ঙ্ঘ্য","ঙ্ঘ্র","ঙ্ম","চ্চ","চ্ছ","চ্ছ্ব","চ্ছ্র","চ্ঞ","চ্ব","চ্য","জ্জ",
        "জ্জ্ব","জ্ঝ","জ্ঞ","জ্ব","জ্য","জ্র","ঞ্চ","ঞ্ছ","ঞ্জ","ঞ্ঝ","ট্ট","ট্ব","ট্ম","ট্য","ট্র","ট্র্য","ড্ড","ড্ব","ড্য","ড্র",
        "ড্র্য","ঢ্য","ঢ্র","ণ্ট","ণ্ঠ","ণ্ঠ্য","ণ্ড","ণ্ড্য","ণ্ড্র","ণ্ঢ","ণ্ণ","ণ্ব","ণ্ম","ণ্য","ত্ত","ত্ত্ব","ত্ত্য","ত্থ","ত্ন","ত্ব",
        "ত্ম","ত্ম্য","ত্য","ত্র","ত্র্য","থ্ব","থ্য","থ্র","থ্র্য","দ্গ","দ্ঘ","দ্দ","দ্দ্ব","দ্ধ","দ্ব","দ্ভ","দ্ভ্র","দ্ম","দ্য","দ্র",
        "দ্র্য","ধ্ন","ধ্ব","ধ্ম","ধ্য","ধ্র","ন্ক","ন্ট","ন্ট্য","ন্ট্র","ন্ট্র্য","ন্ঠ","ন্ড","ন্ড্ব","ন্ড্য","ন্ড্র","ন্ত","ন্ত্ব","ন্ত্য","ন্ত্র",
        "ন্ত্র্য","ন্থ","ন্থ্য","ন্থ্র","ন্দ","ন্দ্ব","ন্দ্য","ন্দ্র","ন্ধ","ন্ধ্য","ন্ধ্র","ন্ন","ন্ব","ন্ম","ন্য","ন্শ্য","ন্স","ন্স্য","প্ট","প্ট্য",
        "প্ত","প্ন","প্প","প্য","প্র","প্র্য","প্ল","প্ল্য","প্স","ফ্য","ফ্র","ফ্র্য","ফ্ল","ফ্ল্য","ব্জ","ব্দ","ব্ধ","ব্ব","ব্য","ব্র",
        "ব্র্য","ব্ল","ভ্ব","ভ্য","ভ্র","ম্ন","ম্ন্য","ম্প","ম্প্য","ম্প্র","ম্ফ","ম্ব","ম্ব্র","ম্ভ","ম্ভ্র","ম্ম","ম্য","ম্র","ম্ল","য্য",
        "র্ক","র্ক্ট","র্ক্য","র্খ","র্গ","র্গ্য","র্গ্র","র্ঘ","র্ঘ্য","র্চ","র্চ্য","র্ছ","র্জ","র্জ্ঞ","র্জ্য","র্ঝ","র্ট","র্ট্য","র্ট্র","র্ড",
        "র্ড্র","র্ঢ্য","র্ণ","র্ণ্য","র্ত","র্ত্ম","র্ত্য","র্ত্র","র্থ","র্থ্য","র্দ","র্দ্ব","র্দ্র","র্ধ","র্ধ্ব","র্ন","র্ন্ড","র্প","র্প্ট","র্প্ল",
        "র্ফ","র্ব","র্ব্য","র্ভ","র্ম","র্ম্থ","র্ম্প","র্ম্য","র্য","র্ল","র্ল্ড","র্ল্য","র্শ","র্শ্ব","র্শ্য","র্ষ","র্ষ্য","র্স","র্স্ট",
        "র্স্ম","র্স্য","র্হ","র্হ্য","র‍্য","ল্ক","ল্ক্য","ল্গ","ল্চ","ল্ট","ল্ট্য","ল্ট্র","ল্ড","ল্ড্য","ল্ড্র","ল্প","ল্ফ","ল্ব","ল্ব্য",
        "ল্ভ","ল্ম","ল্য","ল্ল","শ্চ","শ্ছ","শ্ন","শ্ব","শ্ম","শ্য","শ্র","শ্র্য","শ্ল","ষ্ক","ষ্ক্র","ষ্ট","ষ্ট্য","ষ্ট্র","ষ্ঠ","ষ্ঠ্য",
        "ষ্ণ","ষ্প","ষ্প্র","ষ্ফ","ষ্ব","ষ্ম","ষ্য","স্ক","স্ক্য","স্ক্র","স্ক্র্য","স্খ","স্চ","স্ট","স্ট্য","স্ট্র","স্ট্র্য","স্ত","স্ত্ব","স্ত্য",
        "স্ত্র","স্থ","স্থ্য","স্ন","স্ন্য","স্প","স্প্য","স্প্র","স্প্র্য","স্প্ল","স্প্ল্য","স্ফ","স্ব","স্ম","স্ম্য","স্য","স্র","স্ল","স্ল্য","হ্ণ",
        "হ্ন","হ্ব","হ্ম","হ্য","হ্র","হ্ল","\u{09DF}\u{09CD}\u{09AF}","ব্ল্য","র্ন্ত","ঠ্য","ভ্ল",
    ].into_iter().collect()
});

pub static COMPLEX_ROOTS: Lazy<HashSet<String>> = Lazy::new(|| {
    let mut s = HashSet::new();
    s.insert(" ".to_string());
    for &v in VOWELS { s.insert(v.to_string()); }
    for &c in CONSONANTS_SINGLE { s.insert(c.to_string()); }
    for &n in NUMBERS { s.insert(n.to_string()); }
    for &p in &["!", "\"", "'", "(", ")", ",", "-", ".", "...", ":", ":-", ";", "<", "=", ">", "?", "[", "]", "{", "}", "।", "৷", "–", "—", "\u{201D}", "√"] {
        s.insert(p.to_string());
    }
    s.insert("৳".to_string());
    for &conj in CONJUNCTS.iter() { s.insert(conj.to_string()); }
    s
});
