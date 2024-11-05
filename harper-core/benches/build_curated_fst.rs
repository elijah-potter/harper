use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use harper_core::FstDictionary;
use harper_dictionary_parsing::{
    parse_default_attribute_list, parse_default_word_list, parse_word_list,
};
use hashbrown::HashMap;

fn parse_curated_dictionary(c: &mut Criterion) {
    c.bench_function("parse_curated_dictionary", |b| {
        b.iter(|| {
            let word_list = parse_default_word_list().unwrap();
            let attr_list = parse_default_attribute_list();

            // There will be at _least_ this number of words
            let mut word_map = HashMap::with_capacity(word_list.len());

            attr_list.expand_marked_words(word_list, &mut word_map);
        })
    });
}

fn build_curated_fst(c: &mut Criterion) {
    let word_list = parse_default_word_list().unwrap();
    let attr_list = parse_default_attribute_list();

    // There will be at _least_ this number of words
    let mut word_map = HashMap::with_capacity(word_list.len());

    attr_list.expand_marked_words(word_list, &mut word_map);

    c.bench_function("build_curated_fst", |b| {
        b.iter({
            || {
                let dict = FstDictionary::new(word_map.clone());
            }
        })
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    parse_curated_dictionary(c);
    build_curated_fst(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
