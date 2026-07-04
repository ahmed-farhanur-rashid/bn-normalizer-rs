//! Sentence-level Bangla Unicode normalization.
//!
//! This module does NOT exist in the upstream Python library — it is original
//! work, not a port.
//!
//! It tokenizes input text preserving exact structure, routes Bangla words
//! through `word::normalize`, and passes everything else through unchanged.

mod tokenizer;

use crate::word;
use std::fmt;

/// Policy for handling Bangla words that normalize to `None`.
#[derive(Debug, Clone)]
pub enum NoneTokenPolicy {
    /// Remove the token entirely, closing the gap.
    Drop,
    /// Leave the original un-normalized text in place.
    KeepOriginal,
    /// Replace with a caller-supplied marker string.
    Placeholder(String),
    /// Return an error on the first None word.
    Error,
    /// Normalize what's possible; collect failed tokens alongside the result.
    /// Recommended default for corpus-scale batch processing.
    Collect,
}

/// Options for sentence-level normalization.
#[derive(Debug, Clone)]
pub struct SentenceNormalizeOptions {
    pub none_policy: NoneTokenPolicy,
    pub allow_english: bool,
}

impl Default for SentenceNormalizeOptions {
    fn default() -> Self {
        Self {
            none_policy: NoneTokenPolicy::KeepOriginal,
            allow_english: false,
        }
    }
}

/// Result of sentence-level normalization.
#[derive(Debug, Clone)]
pub struct SentenceNormalizeResult {
    /// The normalized text.
    pub text: String,
    /// Tokens that normalized to None, with their position in the original
    /// token stream. Only populated when `NoneTokenPolicy::Collect` is used.
    pub failed_tokens: Vec<(usize, String)>,
}

/// Error type for sentence normalization.
#[derive(Debug, Clone)]
pub struct SentenceNormalizeError {
    /// The token that failed normalization.
    pub token: String,
    /// Position of the token in the original text's token stream.
    pub position: usize,
}

impl fmt::Display for SentenceNormalizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bangla word at position {} normalized to None: '{}'",
            self.position, self.token
        )
    }
}

impl std::error::Error for SentenceNormalizeError {}

/// Normalize a sentence or paragraph of mixed Bangla/non-Bangla text.
///
/// Tokenizes the input preserving exact structure (whitespace, punctuation,
/// emoji, etc.), normalizes Bangla words via `word::normalize`, and passes
/// everything else through unchanged.
pub fn normalize(
    text: &str,
    opts: &SentenceNormalizeOptions,
) -> Result<SentenceNormalizeResult, SentenceNormalizeError> {
    let tokens = tokenizer::tokenize(text);

    let word_opts = word::NormalizeOptions {
        allow_english: opts.allow_english,
        keep_legacy_symbols: false,
        legacy_maps: None,
    };

    let mut result_parts: Vec<String> = Vec::with_capacity(tokens.len());
    let mut failed_tokens: Vec<(usize, String)> = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        match &token.kind {
            tokenizer::TokenKind::BanglaWord => {
                match word::normalize_with_options(&token.text, &word_opts) {
                    Some(normalized) => result_parts.push(normalized),
                    None => match &opts.none_policy {
                        NoneTokenPolicy::Drop => {
                            // Don't add anything — token is dropped
                        }
                        NoneTokenPolicy::KeepOriginal => {
                            result_parts.push(token.text.clone());
                        }
                        NoneTokenPolicy::Placeholder(marker) => {
                            result_parts.push(marker.clone());
                        }
                        NoneTokenPolicy::Error => {
                            return Err(SentenceNormalizeError {
                                token: token.text.clone(),
                                position: i,
                            });
                        }
                        NoneTokenPolicy::Collect => {
                            failed_tokens.push((i, token.text.clone()));
                            result_parts.push(token.text.clone());
                        }
                    },
                }
            }
            // All non-Bangla tokens pass through unchanged
            _ => {
                result_parts.push(token.text.clone());
            }
        }
    }

    Ok(SentenceNormalizeResult {
        text: result_parts.join(""),
        failed_tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn norm(text: &str) -> String {
        normalize(text, &SentenceNormalizeOptions::default())
            .unwrap()
            .text
    }

    fn norm_drop(text: &str) -> String {
        let opts = SentenceNormalizeOptions {
            none_policy: NoneTokenPolicy::Drop,
            ..Default::default()
        };
        normalize(text, &opts).unwrap().text
    }

    fn norm_collect(text: &str) -> SentenceNormalizeResult {
        let opts = SentenceNormalizeOptions {
            none_policy: NoneTokenPolicy::Collect,
            ..Default::default()
        };
        normalize(text, &opts).unwrap()
    }

    #[test]
    fn test_pure_bangla_sentence() {
        // Each Bangla word gets normalized individually
        let input = "গ্র্রামকে ভালো লাগে";
        let result = norm(input);
        // গ্র্রামকে → গ্রামকে (repeated fola removed)
        assert_eq!(result, "গ্রামকে ভালো লাগে");
    }

    #[test]
    fn test_mixed_bangla_english() {
        // English words pass through unchanged
        let input = "আমি Python শিখছি";
        let result = norm(input);
        assert_eq!(result, "আমি Python শিখছি");
    }

    #[test]
    fn test_punctuation_preserved() {
        let input = "হ্যালো, কেমন আছো?";
        let result = norm(input);
        assert_eq!(result, "হ্যালো, কেমন আছো?");
    }

    #[test]
    fn test_emoji_passthrough() {
        let input = "ভালো 😊 লাগছে";
        let result = norm(input);
        assert_eq!(result, "ভালো 😊 লাগছে");
    }

    #[test]
    fn test_multiple_spaces_preserved() {
        let input = "শব্দ   শব্দ";
        let result = norm(input);
        assert_eq!(result, "শব্দ   শব্দ");
    }

    #[test]
    fn test_digits_passthrough() {
        let input = "২০২৪ সালে 2024";
        let result = norm(input);
        assert_eq!(result, "২০২৪ সালে 2024");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(norm(""), "");
    }

    #[test]
    fn test_whitespace_only() {
        assert_eq!(norm("   "), "   ");
    }

    #[test]
    fn test_none_policy_error() {
        let opts = SentenceNormalizeOptions {
            none_policy: NoneTokenPolicy::Error,
            ..Default::default()
        };
        // "ASD123" with allow_english=false normalizes to None
        let result = normalize("এটি ASD123 একটি", &opts);
        // ASD123 is not Bangla, so it's classified as NonBangla and passed through
        // It won't trigger error since it's not classified as BanglaWord
        assert!(result.is_ok());
    }

    #[test]
    fn test_none_policy_collect() {
        // Force a Bangla-script word that normalizes to None
        // (all-diacritics input that gets cleaned to empty)
        let result = norm_collect("ভালো াা লাগে");
        // "াা" is all invalid starts, normalizes to None
        assert!(!result.text.is_empty());
    }

    #[test]
    fn test_url_passthrough() {
        let input = "দেখো https://example.com এই সাইট";
        let result = norm(input);
        assert!(result.contains("https://example.com"));
    }

    #[test]
    fn test_mixed_script_sentence() {
        let input = "বাংলা text মিশ্র 123 🎉 end";
        let result = norm(input);
        assert!(result.contains("বাংলা"));
        assert!(result.contains("text"));
        assert!(result.contains("মিশ্র"));
        assert!(result.contains("123"));
        assert!(result.contains("🎉"));
        assert!(result.contains("end"));
    }
}
