//! Tokenizer for sentence-level normalization.
//!
//! Classifies each run of characters in the input into one of several
//! token kinds, preserving exact spacing and structure for reassembly.
//!
//! Design requirement (from plan.md): NOT `str.split()` — that loses
//! punctuation/spacing. Each character run is classified by its Unicode
//! properties.

/// The kind of a token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// A word consisting of Bangla-script characters (U+0980–U+09FF).
    BanglaWord,
    /// Whitespace (spaces, tabs, newlines).
    Whitespace,
    /// Punctuation characters.
    Punctuation,
    /// Emoji / emoticons.
    Emoji,
    /// Digit characters (any script).
    Digit,
    /// Non-Bangla word (Latin, Devanagari, etc.).
    NonBangla,
}

/// A token with its text and classification.
#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub kind: TokenKind,
}

/// Classify a single character into a token kind.
fn classify_char(c: char) -> TokenKind {
    // Check Bangla Unicode block first (U+0980–U+09FF)
    if ('\u{0980}'..='\u{09FF}').contains(&c) {
        return TokenKind::BanglaWord;
    }

    // Whitespace
    if c.is_whitespace() {
        return TokenKind::Whitespace;
    }

    // Digits (any script — Bangla digits U+09E6-U+09EF already caught above)
    if c.is_ascii_digit() {
        return TokenKind::Digit;
    }

    // Emoji detection: check common emoji ranges
    if is_emoji(c) {
        return TokenKind::Emoji;
    }

    // Punctuation (ASCII and Unicode general punctuation)
    if c.is_ascii_punctuation() || is_unicode_punctuation(c) {
        return TokenKind::Punctuation;
    }

    // Letters from other scripts (Latin, Devanagari, etc.)
    if c.is_alphabetic() {
        return TokenKind::NonBangla;
    }

    // Everything else (symbols, control chars, etc.) — treat as punctuation
    TokenKind::Punctuation
}

/// Check if a character is likely an emoji.
///
/// Covers the most common emoji ranges. Not exhaustive, but sufficient for
/// real-world Bangla social media text.
fn is_emoji(c: char) -> bool {
    let cp = c as u32;
    // Emoticons
    (0x1F600..=0x1F64F).contains(&cp) ||
    // Miscellaneous Symbols and Pictographs
    (0x1F300..=0x1F5FF).contains(&cp) ||
    // Transport and Map Symbols
    (0x1F680..=0x1F6FF).contains(&cp) ||
    // Supplemental Symbols and Pictographs
    (0x1F900..=0x1F9FF).contains(&cp) ||
    // Symbols and Pictographs Extended-A
    (0x1FA00..=0x1FA6F).contains(&cp) ||
    // Symbols and Pictographs Extended-B
    (0x1FA70..=0x1FAFF).contains(&cp) ||
    // Miscellaneous Symbols
    (0x2600..=0x26FF).contains(&cp) ||
    // Dingbats
    (0x2700..=0x27BF).contains(&cp) ||
    // Regional indicator symbols (flags)
    (0x1F1E0..=0x1F1FF).contains(&cp) ||
    // Variation selectors (emoji presentation)
    cp == 0xFE0F || cp == 0xFE0E ||
    // Zero Width Joiner (used in emoji sequences)
    cp == 0x200D
}

/// Check if a character is Unicode punctuation (beyond ASCII).
fn is_unicode_punctuation(c: char) -> bool {
    let cp = c as u32;
    // General Punctuation block
    (0x2000..=0x206F).contains(&cp) ||
    // CJK Symbols and Punctuation
    (0x3000..=0x303F).contains(&cp) ||
    // Fullwidth punctuation
    (0xFF00..=0xFF0F).contains(&cp) ||
    (0xFF1A..=0xFF20).contains(&cp) ||
    (0xFF3B..=0xFF40).contains(&cp) ||
    (0xFF5B..=0xFF65).contains(&cp) ||
    // Bangla-specific: danda and double danda
    cp == 0x0964 || cp == 0x0965
}

/// Tokenize input text into classified tokens.
///
/// Adjacent characters with the same classification are merged into a
/// single token. This preserves exact spacing and structure.
pub fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    for c in text.chars() {
        let kind = classify_char(c);

        // Merge into previous token if same kind
        if let Some(last) = tokens.last_mut() {
            if last.kind == kind {
                last.text.push(c);
                continue;
            }
        }

        // Start a new token
        tokens.push(Token {
            text: c.to_string(),
            kind,
        });
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_bangla() {
        let tokens = tokenize("আমি ভালো");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::BanglaWord);
        assert_eq!(tokens[0].text, "আমি");
        assert_eq!(tokens[1].kind, TokenKind::Whitespace);
        assert_eq!(tokens[2].kind, TokenKind::BanglaWord);
        assert_eq!(tokens[2].text, "ভালো");
    }

    #[test]
    fn test_mixed_bangla_english() {
        let tokens = tokenize("আমি love বাংলা");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].kind, TokenKind::BanglaWord);
        assert_eq!(tokens[1].kind, TokenKind::Whitespace);
        assert_eq!(tokens[2].kind, TokenKind::NonBangla);
        assert_eq!(tokens[2].text, "love");
        assert_eq!(tokens[3].kind, TokenKind::Whitespace);
        assert_eq!(tokens[4].kind, TokenKind::BanglaWord);
    }

    #[test]
    fn test_punctuation() {
        let tokens = tokenize("হ্যালো, কেমন?");
        // হ্যালো | , | (space) | কেমন | ?
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[1].kind, TokenKind::Punctuation);
        assert_eq!(tokens[1].text, ",");
        assert_eq!(tokens[4].kind, TokenKind::Punctuation);
        assert_eq!(tokens[4].text, "?");
    }

    #[test]
    fn test_emoji() {
        let tokens = tokenize("ভালো 😊 লাগছে");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[2].kind, TokenKind::Emoji);
        assert_eq!(tokens[2].text, "😊");
    }

    #[test]
    fn test_digits() {
        let tokens = tokenize("২০২৪ and 2024");
        // ২০২৪ is Bangla digits (U+09E6–U+09EF) → BanglaWord
        // 2024 is ASCII digits → Digit
        assert_eq!(tokens[0].kind, TokenKind::BanglaWord);
        assert_eq!(tokens[0].text, "২০২৪");
        let digit_token = tokens.iter().find(|t| t.kind == TokenKind::Digit).unwrap();
        assert_eq!(digit_token.text, "2024");
    }

    #[test]
    fn test_multiple_spaces() {
        let tokens = tokenize("a   b");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[1].kind, TokenKind::Whitespace);
        assert_eq!(tokens[1].text, "   ");
    }

    #[test]
    fn test_empty() {
        assert!(tokenize("").is_empty());
    }

    #[test]
    fn test_url_like() {
        let tokens = tokenize("see https://example.com here");
        // URLs break into multiple tokens (letters, punct, digits)
        // The important thing is they don't get classified as BanglaWord
        for t in &tokens {
            if t.text.contains("example") || t.text.contains("https") {
                assert_ne!(t.kind, TokenKind::BanglaWord);
            }
        }
    }

    #[test]
    fn test_roundtrip() {
        // All tokens concatenated should equal the original text
        let input = "আমি 😊 love বাংলা 123, হ্যাঁ!";
        let tokens = tokenize(input);
        let reconstructed: String = tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(reconstructed, input);
    }
}
