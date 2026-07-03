//! # bn-normalize-rs
//!
//! Fast Bangla Unicode text normalization — a faithful Rust port of
//! [bnunicodenormalizer](https://github.com/mnansary/bnUnicodeNormalizer)
//! by Bengali.AI.
//!
//! ## Quick start
//!
//! ```rust
//! use bn_normalize_rs::word;
//!
//! // Normalize a single Bangla word
//! assert_eq!(word::normalize("গ্র্রামকে"), Some("গ্রামকে".to_string()));
//!
//! // Invalid / non-Bangla input returns None
//! assert_eq!(word::normalize("ASD123"), None);
//!
//! // With English allowed
//! let opts = word::NormalizeOptions {
//!     allow_english: true,
//!     ..Default::default()
//! };
//! assert_eq!(word::normalize_with_options("ASD123", &opts), Some("ASD123".to_string()));
//! ```
//!
//! ## Sentence-level normalization
//!
//! ```rust
//! use bn_normalize_rs::sentence;
//!
//! let opts = sentence::SentenceNormalizeOptions::default();
//! let result = sentence::normalize("গ্র্রামকে ভালো লাগে", &opts).unwrap();
//! assert_eq!(result.text, "গ্রামকে ভালো লাগে");
//! ```

pub mod langs;
pub mod word;
pub mod sentence;

mod python;
