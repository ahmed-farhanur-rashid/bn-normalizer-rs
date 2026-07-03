//! Word-level Bangla Unicode normalization.
//!
//! Faithful port of `bnunicodenormalizer.Normalizer.__call__`.

mod ops;

use crate::langs;
use std::collections::{HashMap, HashSet};

/// Options for word normalization, matching the Python constructor args.
#[derive(Debug, Clone)]
pub struct NormalizeOptions {
    pub allow_english: bool,
    pub keep_legacy_symbols: bool,
    /// `None` = no legacy mapping; `Some(map)` = custom char→str map.
    /// Use `default_legacy_maps()` for the upstream default.
    pub legacy_maps: Option<HashMap<char, String>>,
}

impl Default for NormalizeOptions {
    fn default() -> Self {
        Self {
            allow_english: false,
            keep_legacy_symbols: false,
            legacy_maps: None,
        }
    }
}

/// Returns the default legacy maps from the upstream library.
pub fn default_legacy_maps() -> HashMap<char, String> {
    langs::DEFAULT_LEGACY_MAPS
        .iter()
        .map(|(&k, &v)| (k, v.to_string()))
        .collect()
}

/// Normalize a single Bangla word using default options.
///
/// Returns `None` if the word normalizes to empty (invalid/dropped).
pub fn normalize(word: &str) -> Option<String> {
    normalize_with_options(word, &NormalizeOptions::default())
}

/// Normalize a single Bangla word with custom options.
pub fn normalize_with_options(word: &str, opts: &NormalizeOptions) -> Option<String> {
    // Build valid/roots sets based on options
    let mut valid: HashSet<char> = langs::VALID_CHARS.clone();
    let mut roots: HashSet<String> = langs::COMPLEX_ROOTS.clone();

    if opts.allow_english {
        valid.extend(langs::ENGLISH_VALID.iter());
        for c in langs::ENGLISH_VALID.iter() {
            roots.insert(c.to_string());
        }
    }
    if opts.keep_legacy_symbols {
        valid.extend(langs::LEGACY_SYMBOLS.iter());
        for c in langs::LEGACY_SYMBOLS.iter() {
            roots.insert(c.to_string());
        }
    }

    let mut ctx = NormCtx {
        word: word.to_string(),
        decomp: Vec::new(),
        valid,
        roots,
        legacy_maps: opts.legacy_maps.clone(),
    };

    // ── Word-level ops ──
    // LegacySymbols
    ops::map_legacy_symbols(&mut ctx);
    // BrokenDiacritics
    ops::fix_broken_diacritics(&mut ctx);
    // AssameseReplacement
    ops::replace_assamese(&mut ctx);
    // PunctuationReplacement
    ops::replace_punctuations(&mut ctx);

    // ── Decompose ──
    ctx.decomp = ctx.word.chars().map(|c| Some(c.to_string())).collect();

    // ── Decomp-level ops (each via safeop) ──
    safeop(&mut ctx, ops::fix_broken_nukta);
    if ctx.decomp_empty() { return None; }

    safeop(&mut ctx, ops::clean_invalid_unicodes);
    if ctx.decomp_empty() { return None; }

    safeop(&mut ctx, |c| ops::clean_invalid_connector_bangla(c));
    if ctx.decomp_empty() { return None; }

    // FixDiacritics = 4 sub-steps, each via safeop
    safeop(&mut ctx, ops::clean_vowel_diacritics);
    safeop(&mut ctx, ops::clean_consonant_diacritics_bangla);
    safeop(&mut ctx, ops::fix_diacritic_order);
    safeop(&mut ctx, ops::clean_non_char_diacs);
    if ctx.decomp_empty() { return None; }

    // VowelDiacriticAfterVowel (Bangla override)
    safeop(&mut ctx, ops::clean_vowel_diacritic_after_vowel_bangla);
    if ctx.decomp_empty() { return None; }

    // base_bangla_compose
    safeop(&mut ctx, |c| base_compose(c));
    if ctx.decomp_empty() { return None; }

    // ToAndHosontoNormalize
    safeop(&mut ctx, |c| ops::normalize_to_and_hosonto(c));
    if ctx.decomp_empty() { return None; }

    // NormalizeConjunctsDiacritics
    safeop(&mut ctx, |c| ops::clean_invalid_conjunct_diacritics(c));
    if ctx.decomp_empty() { return None; }

    // ComplexRootNormalization
    safeop(&mut ctx, |c| ops::convert_complex_roots(c));
    if ctx.decomp_empty() { return None; }

    // ── Final compose ──
    safeop(&mut ctx, |c| base_compose(c));

    // Join
    let result: String = ctx.decomp.iter()
        .filter_map(|x| x.as_ref())
        .map(|s| s.as_str())
        .collect();

    if result.is_empty() { None } else { Some(result) }
}

/// Internal context for the normalizer (replaces Python `self`).
pub(crate) struct NormCtx {
    pub word: String,
    pub decomp: Vec<Option<String>>,
    pub valid: HashSet<char>,
    pub roots: HashSet<String>,
    pub legacy_maps: Option<HashMap<char, String>>,
}

impl NormCtx {
    pub fn decomp_empty(&self) -> bool {
        self.decomp.iter().all(|x| x.is_none())
    }

    /// Get the string at decomp[idx], or None.
    pub fn get(&self, idx: usize) -> Option<&str> {
        self.decomp.get(idx).and_then(|x| x.as_deref())
    }

    /// Get the single char at decomp[idx] if it's a single-char string.
    pub fn get_char(&self, idx: usize) -> Option<char> {
        self.get(idx).and_then(|s| {
            let mut chars = s.chars();
            let c = chars.next()?;
            if chars.next().is_none() { Some(c) } else { None }
        })
    }
}

/// The `safeop` wrapper: filter None, rejoin+resplit, run op, filter+rejoin+resplit again.
fn safeop(ctx: &mut NormCtx, op: impl FnOnce(&mut NormCtx)) {
    // Pre: filter None, rejoin, resplit
    rejoin_resplit(ctx);
    // Run op
    op(ctx);
    // Post: filter None, rejoin, resplit
    rejoin_resplit(ctx);
}

fn rejoin_resplit(ctx: &mut NormCtx) {
    let joined: String = ctx.decomp.iter()
        .filter_map(|x| x.as_ref())
        .map(|s| s.as_str())
        .collect();
    ctx.decomp = joined.chars().map(|c| Some(c.to_string())).collect();
}

/// `baseCompose` — runs a fixed sub-pipeline.
fn base_compose(ctx: &mut NormCtx) {
    safeop_inner(ctx, ops::clean_invalid_unicodes);
    safeop_inner(ctx, |c| ops::clean_invalid_connector_bangla(c));
    // cleanDiacritics = 4 sub-steps
    safeop_inner(ctx, ops::clean_vowel_diacritics);
    safeop_inner(ctx, ops::clean_consonant_diacritics_bangla);
    safeop_inner(ctx, ops::fix_diacritic_order);
    safeop_inner(ctx, ops::clean_non_char_diacs);
    safeop_inner(ctx, ops::clean_vowel_diacritic_after_vowel_bangla);
    safeop_inner(ctx, |c| ops::fix_no_space_char_bangla(c));
}

/// Inner safeop (used inside base_compose, which is itself inside a safeop).
fn safeop_inner(ctx: &mut NormCtx, op: impl FnOnce(&mut NormCtx)) {
    rejoin_resplit(ctx);
    op(ctx);
    rejoin_resplit(ctx);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_normalization() {
        // Repeated fola: গ্র্রামকে → গ্রামকে
        assert_eq!(normalize("গ্র্রামকে"), Some("গ্রামকে".to_string()));
    }

    #[test]
    fn test_invalid_start() {
        // াটোবাকো → টোবাকো (leading vowel diacritic removed)
        assert_eq!(normalize("াটোবাকো"), Some("টোবাকো".to_string()));
    }

    #[test]
    fn test_english_rejected() {
        assert_eq!(normalize("ASD123"), None);
    }

    #[test]
    fn test_english_allowed() {
        let opts = NormalizeOptions {
            allow_english: true,
            ..Default::default()
        };
        assert_eq!(normalize_with_options("ASD123", &opts), Some("ASD123".to_string()));
    }

    #[test]
    fn test_to_hosonto() {
        // উত্স → উৎস
        assert_eq!(normalize("উত্স"), Some("উৎস".to_string()));
    }

    #[test]
    fn test_double_diacritics() {
        // যুুদ্ধ → যুদ্ধ
        assert_eq!(normalize("যুুদ্ধ"), Some("যুদ্ধ".to_string()));
    }

    #[test]
    fn test_invalid_hosonto() {
        // দুই্টি → দুইটি
        assert_eq!(normalize("দুই্টি"), Some("দুইটি".to_string()));
    }

    #[test]
    fn test_vowel_diacritic_after_vowel() {
        // উুলু → উলু
        assert_eq!(normalize("উুলু"), Some("উলু".to_string()));
    }

    #[test]
    fn test_ending_hosonto() {
        // অজানা্ → অজানা
        assert_eq!(normalize("অজানা্"), Some("অজানা".to_string()));
    }

    #[test]
    fn test_broken_connector() {
        // সং্যুক্তি → সংযুক্তি
        assert_eq!(normalize("সং্যুক্তি"), Some("সংযুক্তি".to_string()));
    }
}
