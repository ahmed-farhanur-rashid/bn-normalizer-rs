//! Individual normalization operations, ported from base.py + normalizer.py.
//!
//! Optimized: decomp is Vec<Option<char>> instead of Vec<Option<String>>.
//! Multi-char insertions are handled by expanding in-place.

use std::collections::HashSet;
use super::NormCtx;
use crate::langs;

// ═══════════════════════════════════════════════════════════════════════
// Word-level ops (operate on ctx.word string directly)
// ═══════════════════════════════════════════════════════════════════════

pub fn map_legacy_symbols(ctx: &mut NormCtx) {
    if let Some(maps) = ctx.legacy_maps {
        for (k, v) in maps {
            ctx.word = ctx.word.replace(&k.to_string(), v);
        }
    }
}

pub fn fix_broken_diacritics(ctx: &mut NormCtx) {
    for &(from, to) in langs::DIACRITIC_MAP {
        ctx.word = ctx.word.replace(from, to);
    }
}

pub fn replace_assamese(ctx: &mut NormCtx) {
    for &(from, to) in langs::ASSAMESE_MAP {
        ctx.word = ctx.word.replace(from, to);
    }
}

pub fn replace_punctuations(ctx: &mut NormCtx) {
    for &(from, to) in langs::PUNCTUATION_MAP {
        ctx.word = ctx.word.replace(from, to);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Decomp-level ops (operate on ctx.decomp: Vec<Option<char>>)
// ═══════════════════════════════════════════════════════════════════════

#[inline]
fn ch(ctx: &NormCtx, idx: usize) -> Option<char> { ctx.get_char(idx) }

// ─── fixBrokenNukta ───────────────────────────────────────────────────
pub fn fix_broken_nukta(ctx: &mut NormCtx) {
    let len = ctx.decomp.len();
    for idx in 0..len {
        if ch(ctx, idx) == Some(langs::NUKTA) {
            if idx > 0 {
                for cidx in (0..idx).rev() {
                    if let Some(c) = ch(ctx, cidx) {
                        if let Some(&replacement) = langs::NUKTA_MAP_STR.get(&c) {
                            // replacement is &str like "\u{09DC}" — always single char
                            ctx.decomp[cidx] = replacement.chars().next().map(|c| c);
                            ctx.decomp[idx] = None;
                            break;
                        }
                    }
                }
            }
        }
    }
}

// ─── cleanInvalidUnicodes ─────────────────────────────────────────────
pub fn clean_invalid_unicodes(ctx: &mut NormCtx) {
    while !ctx.decomp.is_empty() {
        if let Some(c) = ch(ctx, 0) {
            if langs::INVALID_STARTS.contains(&c) { ctx.decomp.remove(0); } else { break; }
        } else { ctx.decomp.remove(0); }
    }
    while !ctx.decomp.is_empty() {
        let last = ctx.decomp.len() - 1;
        if ch(ctx, last) == Some(langs::CONNECTOR) { ctx.decomp.pop(); } else { break; }
    }
    if ctx.decomp.is_empty() { return; }
    for idx in 0..ctx.decomp.len() {
        if let Some(c) = ch(ctx, idx) {
            if !ctx.is_valid(c) { ctx.decomp[idx] = None; }
        }
    }
}

// ─── cleanInvalidConnector (Bangla override) ──────────────────────────
pub fn clean_invalid_connector_bangla(ctx: &mut NormCtx) {
    let len = ctx.decomp.len();
    for idx in 0..len {
        if ch(ctx, idx) == Some(langs::CONNECTOR) && idx < len - 1 {
            let prev = if idx > 0 { ch(ctx, idx - 1) } else { None };
            let next = ch(ctx, idx + 1);
            if let (Some(p), Some(n)) = (prev, next) {
                if n != 'য' && p != 'অ' && p != 'এ' {
                    if langs::INVALID_CONNECTORS.contains(&p) || langs::INVALID_CONNECTORS.contains(&n) {
                        ctx.decomp[idx] = None;
                    }
                }
                if p == 'য' && n != 'য' { ctx.decomp[idx] = None; }
                if p == 'ব' && !['জ', 'দ', 'ধ', 'ব', 'য', 'র', 'ল'].contains(&n) {
                    ctx.decomp[idx] = None;
                }
            }
        }
    }

    // Rebuild word for string-level replacements
    let mut word: String = ctx.decomp.iter().filter_map(|x| *x).collect();
    if word.contains("এ্যা") { word = word.replace("এ্যা", "অ্যা"); }
    if word.contains("অ্য") { word = word.replace("অ্য", "অ্যা"); }
    ctx.decomp = word.chars().map(Some).collect();
}

// ─── cleanVowelDiacritics ─────────────────────────────────────────────
pub fn clean_vowel_diacritics(ctx: &mut NormCtx) {
    let len = ctx.decomp.len();
    for idx in 0..len.saturating_sub(1) {
        if let (Some(d), Some(next)) = (ch(ctx, idx), ch(ctx, idx + 1)) {
            if langs::VOWEL_DIACRITICS_SET.contains(&d) && langs::VOWEL_DIACRITICS_SET.contains(&next) {
                if d == next { ctx.decomp[idx] = None; } else { ctx.decomp[idx + 1] = None; }
            }
        }
    }
}

// ─── cleanConsonantDiacritics (Bangla override) ───────────────────────
pub fn clean_consonant_diacritics_bangla(ctx: &mut NormCtx) {
    let len = ctx.decomp.len();
    for idx in 0..len.saturating_sub(1) {
        if let (Some(d), Some(next)) = (ch(ctx, idx), ch(ctx, idx + 1)) {
            if langs::CONSONANT_DIACRITICS_SET.contains(&d) && langs::CONSONANT_DIACRITICS_SET.contains(&next) {
                if d == next {
                    ctx.decomp[idx] = None;
                } else if (d == 'ং' || d == 'ঃ') && next == 'ঁ' {
                    ctx.decomp[idx] = Some(next);
                    ctx.decomp[idx + 1] = Some(d);
                } else if d == 'ং' && next == 'ঃ' {
                    ctx.decomp[idx + 1] = None;
                } else if d == 'ঃ' && next == 'ং' {
                    ctx.decomp[idx + 1] = None;
                }
            }
        }
    }
}

// ─── fixDiacriticOrder ────────────────────────────────────────────────
pub fn fix_diacritic_order(ctx: &mut NormCtx) {
    let len = ctx.decomp.len();
    for idx in 0..len.saturating_sub(1) {
        if let (Some(d), Some(next)) = (ch(ctx, idx), ch(ctx, idx + 1)) {
            if langs::CONSONANT_DIACRITICS_SET.contains(&d) && langs::VOWEL_DIACRITICS_SET.contains(&next) {
                ctx.decomp[idx] = Some(next);
                ctx.decomp[idx + 1] = Some(d);
            }
        }
    }
}

// ─── cleanNonCharDiacs ────────────────────────────────────────────────
pub fn clean_non_char_diacs(ctx: &mut NormCtx) {
    for idx in 1..ctx.decomp.len() {
        if let (Some(d), Some(prev)) = (ch(ctx, idx), ch(ctx, idx - 1)) {
            if langs::DIACRITICS.contains(&d) && langs::NON_CHARS.contains(&prev) {
                ctx.decomp[idx] = None;
            }
        }
    }
}

// ─── cleanVowelDiacriticComingAfterVowel (Bangla override) ────────────
pub fn clean_vowel_diacritic_after_vowel_bangla(ctx: &mut NormCtx) {
    // This op can produce multi-char expansion: এ → ত্র (3 chars replacing 1).
    // We need to collect expansions and apply them after.
    let mut expansions: Vec<(usize, Vec<char>)> = Vec::new();

    for idx in 0..ctx.decomp.len() {
        if let Some(d) = ch(ctx, idx) {
            if langs::VOWEL_DIACRITICS_SET.contains(&d) && idx > 0 {
                if let Some(prev) = ch(ctx, idx - 1) {
                    if langs::VOWELS_SET.contains(&prev) {
                        if prev != 'এ' {
                            ctx.decomp[idx] = None;
                        } else {
                            // এ → ত্র (3 chars replacing 1 at idx-1)
                            expansions.push((idx - 1, vec!['ত', '্', 'র']));
                        }
                    }
                }
            }
        }
    }

    // Apply expansions in reverse order to maintain indices
    for (idx, chars) in expansions.into_iter().rev() {
        ctx.decomp[idx] = Some(chars[0]);
        // Insert remaining chars after idx
        for (i, &c) in chars[1..].iter().enumerate() {
            ctx.decomp.insert(idx + 1 + i, Some(c));
        }
    }
}

// ─── fixNoSpaceChar (Bangla override) ─────────────────────────────────
pub fn fix_no_space_char_bangla(ctx: &mut NormCtx) {
    // Phase 1
    for idx in 0..ctx.decomp.len() {
        if let Some(c) = ch(ctx, idx) {
            if idx == 0 && (c == '\u{200C}' || c == '\u{200D}') {
                ctx.decomp[idx] = None;
            } else if c == '\u{200C}' {
                ctx.decomp[idx] = Some('\u{200D}');
            }
        }
    }
    ctx.decomp.retain(|x| x.is_some());

    // Phase 2
    let len = ctx.decomp.len();
    // Track which indices to merge র+ZWJ
    let mut merge_indices: Vec<usize> = Vec::new();

    for idx in 1..len {
        if ch(ctx, idx) == Some('\u{200D}') {
            if idx == len - 1 {
                ctx.decomp[idx] = None;
            } else {
                let prev = ch(ctx, idx - 1);
                if prev == Some(langs::CONNECTOR) {
                    ctx.decomp[idx] = None;
                    ctx.decomp[idx - 1] = None;
                } else if prev != Some('র') {
                    ctx.decomp[idx] = None;
                } else {
                    if idx > 1 && ch(ctx, idx - 2) == Some(langs::CONNECTOR) {
                        ctx.decomp[idx] = None;
                    } else if idx < len - 1 && ch(ctx, idx + 1) != Some(langs::CONNECTOR) {
                        ctx.decomp[idx] = None;
                    } else if idx < len - 2 && ch(ctx, idx + 2) != Some('য') && ch(ctx, idx + 1) != Some(langs::CONNECTOR) {
                        ctx.decomp[idx] = None;
                    } else {
                        // Merge র + ZWJ: mark for post-processing
                        // In the original, this produces a multi-char string "র‍"
                        // We keep both chars; the rejoin_resplit will handle it
                        merge_indices.push(idx);
                    }
                }
            }
        }
    }
    ctx.decomp.retain(|x| x.is_some());
}

// ─── To+Hosonto normalization ─────────────────────────────────────────

fn convert_to_and_hosonto(ctx: &mut NormCtx) {
    let len = ctx.decomp.len();
    for idx in 0..len {
        if idx < len - 1 && ch(ctx, idx) == Some('ত') && ch(ctx, idx + 1) == Some(langs::CONNECTOR) {
            if idx < len - 2 {
                if let Some(n2) = ch(ctx, idx + 2) {
                    if !langs::VALID_AFTER_TO_HOSONTO.contains(&n2) {
                        ctx.decomp[idx] = Some('ৎ');
                        ctx.decomp[idx + 1] = None;
                    } else if idx < len - 3 && n2 == 'ত' && ch(ctx, idx + 3) == Some(langs::CONNECTOR) {
                        if idx < len - 4 {
                            if let Some(n4) = ch(ctx, idx + 4) {
                                if !['ব', 'য', 'র'].contains(&n4) {
                                    ctx.decomp[idx] = Some('ৎ');
                                    ctx.decomp[idx + 1] = None;
                                }
                                if n4 == 'র' { ctx.decomp[idx + 3] = None; }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn swap_to_and_hosonto_diacritics(ctx: &mut NormCtx) {
    let len = ctx.decomp.len();
    for idx in 0..len.saturating_sub(1) {
        if ch(ctx, idx) == Some('ৎ') {
            if let Some(next) = ch(ctx, idx + 1) {
                if langs::DIACRITICS.contains(&next) {
                    ctx.decomp.swap(idx, idx + 1);
                }
            }
        }
    }
}

pub fn normalize_to_and_hosonto(ctx: &mut NormCtx) {
    safeop_inner(ctx, convert_to_and_hosonto);
    safeop_inner(ctx, swap_to_and_hosonto_diacritics);
    base_compose_inner(ctx);
}

// ─── Conjunct diacritics cleanup ──────────────────────────────────────

fn fix_typo_for_jo_fola(ctx: &mut NormCtx) {
    let len = ctx.decomp.len();
    for idx in 0..len.saturating_sub(1) {
        if ch(ctx, idx) == Some(langs::CONNECTOR) {
            // The installed Python library uses U+09DF (pre-composed য়)
            if ch(ctx, idx + 1) == Some('\u{09DF}') {
                ctx.decomp[idx + 1] = Some('\u{09AF}');
            }
        }
    }
}

fn clean_double_cc(ctx: &mut NormCtx) {
    let len = ctx.decomp.len();
    for idx in 0..len.saturating_sub(3) {
        if ch(ctx, idx) == Some(langs::CONNECTOR) && ch(ctx, idx + 2) == Some(langs::CONNECTOR) {
            if let (Some(c1), Some(c2)) = (ch(ctx, idx + 1), ch(ctx, idx + 3)) {
                if langs::CONSONANTS_SINGLE_SET.contains(&c1) && langs::CONSONANTS_SINGLE_SET.contains(&c2) && c1 == c2 {
                    ctx.decomp[idx] = None;
                    ctx.decomp[idx + 1] = None;
                }
            }
        }
    }
}

fn clean_double_ref(ctx: &mut NormCtx) {
    let len = ctx.decomp.len();
    for idx in 0..len.saturating_sub(3) {
        if ch(ctx, idx) == Some('র') && ch(ctx, idx + 1) == Some(langs::CONNECTOR)
            && ch(ctx, idx + 2) == Some('র') && ch(ctx, idx + 3) == Some(langs::CONNECTOR)
        {
            ctx.decomp[idx] = None;
            ctx.decomp[idx + 1] = None;
        }
    }
}

fn clean_connector_for_jo_fola(ctx: &mut NormCtx) {
    let len = ctx.decomp.len();
    for idx in 0..len.saturating_sub(2) {
        if ch(ctx, idx) == Some(langs::CONNECTOR) && ch(ctx, idx + 1) == Some('য') && ch(ctx, idx + 2) == Some(langs::CONNECTOR) {
            ctx.decomp[idx + 2] = None;
        }
    }
}

pub fn clean_invalid_conjunct_diacritics(ctx: &mut NormCtx) {
    safeop_inner(ctx, fix_typo_for_jo_fola);
    safeop_inner(ctx, clean_double_cc);
    safeop_inner(ctx, clean_double_ref);
    safeop_inner(ctx, clean_connector_for_jo_fola);
    base_compose_inner(ctx);
}

// ─── Complex roots ────────────────────────────────────────────────────

fn construct_complex_decomp(ctx: &mut NormCtx) {
    let conn = langs::CONNECTOR;
    let zwj = '\u{200D}';
    if !ctx.decomp.iter().any(|x| *x == Some(conn)) { return; }

    let c_idxs: Vec<usize> = ctx.decomp.iter().enumerate()
        .filter(|&(_, x)| *x == Some(conn))
        .map(|(i, _)| i).collect();

    let mut comps: Vec<Vec<usize>> = c_idxs.iter()
        .filter_map(|&cid| {
            if cid > 0 && cid < ctx.decomp.len() - 1 {
                let mut group = vec![cid - 1, cid, cid + 1];
                // If the char before the group start is ZWJ, include it and the char before that
                // This handles র‍্য where ZWJ is between র and ্
                if cid >= 2 && ch(ctx, cid - 1) == Some(zwj) {
                    // ZWJ is at cid-1, so include cid-2 (the র)
                    group.insert(0, cid - 2);
                }
                Some(group)
            } else { None }
        }).collect();

    let mut r_decomp: Vec<Vec<usize>> = Vec::new();
    while !comps.is_empty() {
        let mut first: HashSet<usize> = comps.remove(0).into_iter().collect();
        loop {
            let prev_len = first.len();
            let mut rest = Vec::new();
            for r in comps.drain(..) {
                if first.intersection(&r.iter().copied().collect()).count() > 0 { first.extend(r); }
                else { rest.push(r); }
            }
            comps = rest;
            if first.len() == prev_len { break; }
        }
        let mut sorted: Vec<usize> = first.into_iter().collect();
        sorted.sort();
        r_decomp.push(sorted);
    }

    // Build combined strings and replace decomp entries.
    for ridx in &r_decomp {
        let mut comb = String::new();
        for &i in ridx {
            if let Some(c) = ctx.decomp[i] { comb.push(c); }
        }
        let chars: Vec<char> = comb.chars().collect();
        for (j, &i) in ridx.iter().enumerate() {
            if j < chars.len() {
                ctx.decomp[i] = Some(chars[j]);
            } else {
                ctx.decomp[i] = None;
            }
        }
    }

    ctx.decomp.retain(|x| x.is_some());
}

fn check_complex_root(root: &str, ctx: &NormCtx) -> String {
    let chars: Vec<char> = root.chars().collect();
    let mut formed: Vec<String> = Vec::new();
    let mut formed_idx: HashSet<usize> = HashSet::new();

    let mut i = 0;
    while i < chars.len() {
        if chars[i] != langs::CONNECTOR && !formed_idx.contains(&i) {
            let mut r = chars[i].to_string();
            if i == chars.len() - 1 {
                formed.push(r);
                i += 1;
                continue;
            }
            let mut j = i + 2;
            let mut found_end = false;
            while j < chars.len() {
                let d = chars[j];
                let k = format!("{}\u{09CD}{}", r, d);
                if !ctx.is_valid_root(&k) {
                    formed.push(r.clone());
                    found_end = true;
                    break;
                } else if j != chars.len() - 1 {
                    r = k;
                    formed_idx.insert(j);
                    j += 2;
                } else {
                    r = k;
                    formed_idx.insert(j);
                    formed.push(r.clone());
                    found_end = true;
                    break;
                }
            }
            if !found_end { formed.push(r.clone()); }
        }
        i += 1;
    }
    formed.join("")
}

pub fn convert_complex_roots(ctx: &mut NormCtx) {
    fix_no_space_char_bangla(ctx);
    ctx.decomp.retain(|x| x.is_some());
    construct_complex_decomp(ctx);

    // Now decomp is a clean Vec<Option<char>>. We need to identify conjunct groups
    // (sequences connected by CONNECTOR) and validate them as complex roots.
    // Rebuild decomp as a flat string, identify connector-bound groups, check them.
    let flat: String = ctx.decomp.iter().filter_map(|x| *x).collect();
    let chars: Vec<char> = flat.chars().collect();

    // Find conjunct group boundaries (same logic as construct_complex_decomp but simpler)
    let conn = langs::CONNECTOR;
    if !chars.contains(&conn) { return; }

    // Re-decompose into groups (must match construct_complex_decomp logic)
    let zwj = '\u{200D}';
    let c_idxs: Vec<usize> = chars.iter().enumerate()
        .filter(|&(_, &c)| c == conn)
        .map(|(i, _)| i).collect();

    let mut comps: Vec<Vec<usize>> = c_idxs.iter()
        .filter_map(|&cid| {
            if cid > 0 && cid < chars.len() - 1 {
                let mut group = vec![cid - 1, cid, cid + 1];
                // Include ZWJ chars preceding the group start
                if cid >= 2 && chars[cid - 1] == zwj {
                    group.insert(0, cid - 2);
                }
                Some(group)
            } else { None }
        }).collect();

    let mut r_decomp: Vec<Vec<usize>> = Vec::new();
    while !comps.is_empty() {
        let mut first: HashSet<usize> = comps.remove(0).into_iter().collect();
        loop {
            let prev_len = first.len();
            let mut rest = Vec::new();
            for r in comps.drain(..) {
                if first.intersection(&r.iter().copied().collect()).count() > 0 { first.extend(r); }
                else { rest.push(r); }
            }
            comps = rest;
            if first.len() == prev_len { break; }
        }
        let mut sorted: Vec<usize> = first.into_iter().collect();
        sorted.sort();
        r_decomp.push(sorted);
    }

    // Check each conjunct group
    let mut result_chars: Vec<Option<char>> = chars.iter().map(|&c| Some(c)).collect();
    for ridx in &r_decomp {
        let group: String = ridx.iter().map(|&i| chars[i]).collect();
        if !ctx.is_valid_root(&group) && group.contains(conn) {
            let checked = check_complex_root(&group, ctx);
            let new_chars: Vec<char> = checked.chars().collect();
            // Replace the group in result_chars
            for (j, &i) in ridx.iter().enumerate() {
                if j < new_chars.len() {
                    result_chars[i] = Some(new_chars[j]);
                } else {
                    result_chars[i] = None;
                }
            }
            // If new_chars is longer than ridx (expansion), we need to insert extras
            if new_chars.len() > ridx.len() {
                // This is very rare; handle by rebuilding
                let last_idx = *ridx.last().unwrap();
                for j in ridx.len()..new_chars.len() {
                    result_chars.insert(last_idx + 1 + (j - ridx.len()), Some(new_chars[j]));
                }
            }
        }
    }

    ctx.decomp = result_chars;
}

// ─── Internal helpers ─────────────────────────────────────────────────

fn rejoin_resplit(ctx: &mut NormCtx) {
    ctx.decomp.retain(|x| x.is_some());
}

fn safeop_inner(ctx: &mut NormCtx, op: impl FnOnce(&mut NormCtx)) {
    rejoin_resplit(ctx);
    op(ctx);
    rejoin_resplit(ctx);
}

fn base_compose_inner(ctx: &mut NormCtx) {
    safeop_inner(ctx, clean_invalid_unicodes);
    safeop_inner(ctx, clean_invalid_connector_bangla);
    safeop_inner(ctx, clean_vowel_diacritics);
    safeop_inner(ctx, clean_consonant_diacritics_bangla);
    safeop_inner(ctx, fix_diacritic_order);
    safeop_inner(ctx, clean_non_char_diacs);
    safeop_inner(ctx, clean_vowel_diacritic_after_vowel_bangla);
    safeop_inner(ctx, fix_no_space_char_bangla);
}
