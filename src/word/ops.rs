//! Individual normalization operations, ported from base.py + normalizer.py.

use std::collections::HashSet;
use super::NormCtx;
use crate::langs;

// ═══════════════════════════════════════════════════════════════════════
// Word-level ops (operate on ctx.word string directly)
// ═══════════════════════════════════════════════════════════════════════

pub fn map_legacy_symbols(ctx: &mut NormCtx) {
    if let Some(ref maps) = ctx.legacy_maps {
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
// Decomp-level ops (operate on ctx.decomp: Vec<Option<String>>)
// ═══════════════════════════════════════════════════════════════════════

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
                            ctx.decomp[cidx] = Some(replacement.to_string());
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
            if !ctx.valid.contains(&c) { ctx.decomp[idx] = None; }
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

    let mut word: String = ctx.decomp.iter().filter_map(|x| x.as_ref()).map(|s| s.as_str()).collect();
    if word.contains("এ্যা") { word = word.replace("এ্যা", "অ্যা"); }
    if word.contains("অ্য") { word = word.replace("অ্য", "অ্যা"); }
    ctx.decomp = word.chars().map(|c| Some(c.to_string())).collect();
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
                    ctx.decomp[idx] = Some(next.to_string());
                    ctx.decomp[idx + 1] = Some(d.to_string());
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
                ctx.decomp[idx] = Some(next.to_string());
                ctx.decomp[idx + 1] = Some(d.to_string());
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
    for idx in 0..ctx.decomp.len() {
        if let Some(d) = ch(ctx, idx) {
            if langs::VOWEL_DIACRITICS_SET.contains(&d) && idx > 0 {
                if let Some(prev) = ch(ctx, idx - 1) {
                    if langs::VOWELS_SET.contains(&prev) {
                        if prev != 'এ' { ctx.decomp[idx] = None; }
                        else { ctx.decomp[idx - 1] = Some("ত্র".to_string()); }
                    }
                }
            }
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
                ctx.decomp[idx] = Some("\u{200D}".to_string());
            }
        }
    }
    ctx.decomp.retain(|x| x.is_some());

    // Phase 2
    let len = ctx.decomp.len();
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
                        if let Some(ref prev_s) = ctx.decomp[idx - 1] {
                            let merged = format!("{}\u{200D}", prev_s);
                            ctx.decomp[idx - 1] = Some(merged);
                            ctx.decomp[idx] = None;
                        }
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
                        ctx.decomp[idx] = Some("ৎ".to_string());
                        ctx.decomp[idx + 1] = None;
                    } else if idx < len - 3 && n2 == 'ত' && ch(ctx, idx + 3) == Some(langs::CONNECTOR) {
                        if idx < len - 4 {
                            if let Some(n4) = ch(ctx, idx + 4) {
                                if !['ব', 'য', 'র'].contains(&n4) {
                                    ctx.decomp[idx] = Some("ৎ".to_string());
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
                    let tmp = ctx.decomp[idx].clone();
                    ctx.decomp[idx] = ctx.decomp[idx + 1].clone();
                    ctx.decomp[idx + 1] = tmp;
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
            // for this comparison, not the decomposed form.
            if ch(ctx, idx + 1) == Some('\u{09DF}') {
                ctx.decomp[idx + 1] = Some("\u{09AF}".to_string());
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
    let conn = langs::CONNECTOR.to_string();
    if !ctx.decomp.iter().any(|x| x.as_deref() == Some(conn.as_str())) { return; }

    let c_idxs: Vec<usize> = ctx.decomp.iter().enumerate()
        .filter(|(_, x)| x.as_deref() == Some(conn.as_str()))
        .map(|(i, _)| i).collect();

    let mut comps: Vec<Vec<usize>> = c_idxs.iter()
        .filter_map(|&cid| {
            if cid > 0 && cid < ctx.decomp.len() - 1 { Some(vec![cid - 1, cid, cid + 1]) } else { None }
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

    for ridx in &r_decomp {
        let mut comb = String::new();
        for &i in ridx {
            if let Some(ref s) = ctx.decomp[i] { comb.push_str(s); }
        }
        for (j, &i) in ridx.iter().enumerate() {
            if j == ridx.len() - 1 { ctx.decomp[i] = Some(comb.clone()); }
            else { ctx.decomp[i] = None; }
        }
    }
    ctx.decomp.retain(|x| x.is_some());
}

fn check_complex_root(root: &str, complex_roots: &HashSet<String>) -> String {
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
                if !complex_roots.contains(&k) {
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

    let roots = ctx.roots.clone();
    let conn = langs::CONNECTOR.to_string();
    for idx in 0..ctx.decomp.len() {
        if let Some(ref d) = ctx.decomp[idx] {
            if !roots.contains(d) && d.contains(conn.as_str()) {
                let checked = check_complex_root(d, &roots);
                ctx.decomp[idx] = Some(checked);
            }
        }
    }
}

// ─── Internal helpers ─────────────────────────────────────────────────

fn rejoin_resplit(ctx: &mut NormCtx) {
    let joined: String = ctx.decomp.iter().filter_map(|x| x.as_ref()).map(|s| s.as_str()).collect();
    ctx.decomp = joined.chars().map(|c| Some(c.to_string())).collect();
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
