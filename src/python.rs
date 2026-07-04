//! PyO3 bindings for bn-normalize-rs.
//!
//! Exposes word-level and sentence-level normalization to Python.

use pyo3::prelude::*;

use crate::word;
use crate::sentence;

/// Normalize a single Bangla word (default options).
///
/// Returns the normalized word, or None if the word is invalid/dropped.
///
/// Example:
///     >>> from bn_normalize_rs import normalize_word
///     >>> normalize_word("গ্র্রামকে")
///     'গ্রামকে'
///     >>> normalize_word("ASD123") is None
///     True
#[pyfunction]
fn normalize_word(word: &str) -> Option<String> {
    word::normalize(word)
}

/// Normalize a single Bangla word with custom options.
///
/// Args:
///     word: The word to normalize.
///     allow_english: If True, English characters are treated as valid.
///     keep_legacy_symbols: If True, legacy Bangla symbols are preserved.
///
/// Returns:
///     The normalized word, or None if the word is invalid/dropped.
#[pyfunction]
#[pyo3(signature = (word, allow_english=false, keep_legacy_symbols=false))]
fn normalize_word_with_options(
    word: &str,
    allow_english: bool,
    keep_legacy_symbols: bool,
) -> Option<String> {
    let opts = word::NormalizeOptions {
        allow_english,
        keep_legacy_symbols,
        legacy_maps: None,
    };
    word::normalize_with_options(word, &opts)
}

/// Normalize a sentence or paragraph of mixed Bangla/non-Bangla text.
///
/// Tokenizes the input preserving exact structure, normalizes Bangla words
/// via `normalize_word`, and passes everything else through unchanged.
///
/// Args:
///     text: The input text to normalize.
///     none_policy: What to do when a Bangla word normalizes to None.
///         One of: "drop", "keep_original", "error", "collect", "drop_and_collect".
///         Default: "keep_original".
///     allow_english: If True, English characters are treated as valid in word normalization.
///
/// Returns:
///     The normalized text string.
///     If none_policy="collect", returns a tuple of (normalized_text, list_of_failed_tokens).
///     If none_policy="error", raises ValueError on the first None word.
#[pyfunction]
#[pyo3(signature = (text, none_policy="keep_original", allow_english=false))]
fn normalize_sentence(
    text: &str,
    none_policy: &str,
    allow_english: bool,
) -> PyResult<PyObject> {
    let policy = match none_policy {
        "drop" => sentence::NoneTokenPolicy::Drop,
        "keep_original" => sentence::NoneTokenPolicy::KeepOriginal,
        "error" => sentence::NoneTokenPolicy::Error,
        "collect" => sentence::NoneTokenPolicy::Collect,
        "drop_and_collect" => sentence::NoneTokenPolicy::DropAndCollect,
        other => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid none_policy '{}'. Must be one of: drop, keep_original, error, collect, drop_and_collect",
                other
            )));
        }
    };

    let opts = sentence::SentenceNormalizeOptions {
        none_policy: policy,
        allow_english,
    };

    let result = sentence::normalize(text, &opts);

    Python::with_gil(|py| {
        match result {
            Ok(res) => {
                if res.failed_tokens.is_empty() {
                    Ok(res.text.into_pyobject(py)?.into_any().unbind())
                } else {
                    let failed: Vec<(usize, String)> = res.failed_tokens;
                    let tuple = (res.text, failed);
                    Ok(tuple.into_pyobject(py)?.into_any().unbind())
                }
            }
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    })
}

/// Normalize a batch of words efficiently.
///
/// Args:
///     words: A list of words to normalize.
///     allow_english: If True, English characters are treated as valid.
///
/// Returns:
///     A list of (word, normalized_or_none) tuples.
#[pyfunction]
#[pyo3(signature = (words, allow_english=false))]
fn normalize_batch(words: Vec<String>, allow_english: bool) -> Vec<(String, Option<String>)> {
    let opts = word::NormalizeOptions {
        allow_english,
        keep_legacy_symbols: false,
        legacy_maps: None,
    };
    words
        .into_iter()
        .map(|w| {
            let normalized = word::normalize_with_options(&w, &opts);
            (w, normalized)
        })
        .collect()
}

/// Fast Bangla Unicode text normalization.
///
/// A faithful Rust port of bnunicodenormalizer by Bengali.AI,
/// with an additional sentence-level normalization module.
#[pymodule]
fn bn_normalize_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(normalize_word, m)?)?;
    m.add_function(wrap_pyfunction!(normalize_word_with_options, m)?)?;
    m.add_function(wrap_pyfunction!(normalize_sentence, m)?)?;
    m.add_function(wrap_pyfunction!(normalize_batch, m)?)?;
    Ok(())
}
