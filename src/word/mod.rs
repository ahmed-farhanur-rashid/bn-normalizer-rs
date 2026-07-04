//! Word-level Bangla Unicode normalization.
//!
//! Faithful port of `bnunicodenormalizer.Normalizer.__call__`.

mod ops;

use crate::langs;
use std::collections::HashMap;

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
    let mut ctx = NormCtx {
        word: word.to_string(),
        decomp: Vec::new(),
        allow_english: opts.allow_english,
        keep_legacy: opts.keep_legacy_symbols,
        legacy_maps: opts.legacy_maps.as_ref(),
    };

    // ── Word-level ops ──
    ops::map_legacy_symbols(&mut ctx);
    ops::fix_broken_diacritics(&mut ctx);
    ops::replace_assamese(&mut ctx);
    ops::replace_punctuations(&mut ctx);

    // ── Decompose ──
    ctx.decomp = ctx.word.chars().map(Some).collect();

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
        .collect::<String>();

    if result.is_empty() { None } else { Some(result) }
}

/// Internal context for the normalizer (replaces Python `self`).
/// Uses references to static sets instead of cloning per call.
pub(crate) struct NormCtx<'a> {
    pub word: String,
    /// Decomposition buffer — `Option<char>` for the common case.
    /// Multi-char strings from ops are handled by `rejoin_resplit` normalizing
    /// everything back to single chars after each op.
    pub decomp: Vec<Option<char>>,
    pub allow_english: bool,
    pub keep_legacy: bool,
    pub legacy_maps: Option<&'a HashMap<char, String>>,
}

impl<'a> NormCtx<'a> {
    pub fn decomp_empty(&self) -> bool {
        self.decomp.iter().all(|x| x.is_none())
    }

    /// Check if a character is valid given current options.
    #[inline]
    pub fn is_valid(&self, c: char) -> bool {
        langs::VALID_CHARS.contains(&c)
            || (self.allow_english && langs::ENGLISH_VALID.contains(&c))
            || (self.keep_legacy && langs::LEGACY_SYMBOLS_SET.contains(&c))
    }

    /// Check if a string is a valid root given current options.
    #[inline]
    pub fn is_valid_root(&self, s: &str) -> bool {
        langs::COMPLEX_ROOTS.contains(s)
            || (self.allow_english && s.len() == 1 && {
                let c = s.chars().next().unwrap();
                langs::ENGLISH_VALID.contains(&c)
            })
            || (self.keep_legacy && s.len() == 1 && {
                let c = s.chars().next().unwrap();
                langs::LEGACY_SYMBOLS_SET.contains(&c)
            })
    }

    /// Get the char at decomp[idx], or None.
    #[inline]
    pub fn get_char(&self, idx: usize) -> Option<char> {
        self.decomp.get(idx).and_then(|x| *x)
    }
}

/// The `safeop` wrapper: filter None, rejoin+resplit, run op, filter+rejoin+resplit again.
fn safeop(ctx: &mut NormCtx, op: impl FnOnce(&mut NormCtx)) {
    rejoin_resplit(ctx);
    op(ctx);
    rejoin_resplit(ctx);
}

/// Filter None entries — the new rejoin_resplit is just a retain since decomp is Vec<Option<char>>.
/// The original Python does: join chars → re-split into chars. With Option<char>, this is
/// equivalent to just filtering out Nones (no multi-char strings to re-split).
fn rejoin_resplit(ctx: &mut NormCtx) {
    ctx.decomp.retain(|x| x.is_some());
}

/// `baseCompose` — runs a fixed sub-pipeline.
fn base_compose(ctx: &mut NormCtx) {
    safeop_inner(ctx, ops::clean_invalid_unicodes);
    safeop_inner(ctx, |c| ops::clean_invalid_connector_bangla(c));
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
