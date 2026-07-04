use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;

fn bench_word_normalize(c: &mut Criterion) {
    let corpus = fs::read_to_string("tests/data/corpus_sample.txt")
        .expect("tests/data/corpus_sample.txt not found");
    let words: Vec<&str> = corpus.lines().collect();
    let count = words.len();

    c.bench_function(&format!("normalize_{}_words", count), |b| {
        b.iter(|| {
            for &word in &words {
                black_box(bn_normalize_rs::word::normalize(word));
            }
        })
    });
}

criterion_group!(benches, bench_word_normalize);
criterion_main!(benches);
